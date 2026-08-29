// arm_grid.cpp — lane w-globarms (L4 of docs/ADOPTION_BRIEF_2026-08-29.md).
//
// GATE A'S TWELVE ARMS, and which of them a real obj can decide.
//
// `docs/whitebox/ref/P_GLOBREGS.md` §3 carries gate A as a twelve-row table of
// tests over the symbol KIND byte at `sym+0x04`.  `w-globobj` reported the
// population exists and did not pursue it.  This grid is the obj half.
//
// THE READ THIS GRID DEPENDS ON, and which `grade_globarms.py` re-derives from
// the pinned image rather than taking from this comment:
//
//   `FUN_10bd2913` (`0x10bd2913`) is the front-end -> back-end symbol map:
//   given a `.gl` record it computes the globregs KIND into `bl` and writes it
//   at `0x10bd2a1d`.  The computation is a dec-chain on the `.gl` record kind
//   `[gl+0x30]` (P_SYMBOL.md §1: 1 data, 3 function, 4 extern/alias) and, for
//   `[gl+0x30] == 1`, an 8-entry jump table at `0x10bd2a9f` indexed by the
//   3-bit LINKAGE field `([gl+0x37] >> 0x15) & 7` — the same field
//   `P_SYMBOL.md` §3 reads at `0x10b28bb4`, where linkage 1 and 3 are the
//   classes that produce NO COFF RECORD AT ALL.
//
//       linkage 1  -> kind 4          } A6: eligible
//       linkage 3  -> kind 5          }
//       linkage 5  -> kind 5 or 7
//       linkage 2,6 -> kind 7 or 8    } A8: eligible, ALWAYS aliased
//       linkage 4,7 -> kind 7, 8 or 9 } A8 / A9
//       [gl+0x30]==3 (a function) -> kind 0xb  -> A9 REJECT
//       [gl+0x30]==4 (extern/alias) -> kind 0xa -> A10
//
//   So **linkage 1 and 3 — the no-COFF-record classes, i.e. ordinary autos —
//   are exactly the kinds A6 admits**, and a symbol that gets a COFF record is
//   a kind-7/8/9 symbol reaching A8 or A9.
//
// THE READOUT is `w-globobj`'s frame-traffic rule, re-implemented here rather
// than imported, and cross-checked against `grade_globobj.py --promote` on the
// same dumps (prereg §5 control C4).  A promoted local needs no stack slot; the
// prologue's own saves sit BEFORE the `stwu` and are excluded by construction.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globarms/arm_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W, the workload profile)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globarms/arm_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X, the fixture profile)
// Grade:    docs/whitebox/scripts/grade_globarms.py --arms <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);
extern "C" void u_p(int *);
extern "C" int f1(int);
extern "C" int g1(int);
extern "C" int h1(int);

// ---- A6, kinds 4/5: an auto whose address is NOT taken -------------------
// linkage 1 or 3 -> kind 4 or 5 -> A6 eligible, and `sym+0x05 & 2` clear so it
// does NOT join the DAT_10c2e3e8 aliasing set.  POSITIVE CONTROL.
extern "C" int ga_int(int *p) {
    int x = p[0];
    u_i(sink(1));
    return x;
}

// ---- NEGATIVE CONTROL: volatile ------------------------------------------
extern "C" int ga_vol(int *p) {
    volatile int x = p[0];
    u_i(sink(1));
    return x;
}

// ---- A6's sub-branch: `sym+0x05 & 2` set, i.e. the address escapes --------
// Same kind (4 or 5), same arm, DIFFERENT side of the arm's internal test.
extern "C" int ga_escape(int *p) {
    int x = p[0];
    u_p(&x);
    u_i(sink(1));
    return x;
}

// ---- A8, kinds 7/8: symbols that DO get a COFF record --------------------
extern int ge_extern;
static int gs_fstatic;

extern "C" int ga_extern(int *p) {
    ge_extern = p[0];
    u_i(sink(1));
    return ge_extern;
}

extern "C" int ga_fstatic(int *p) {
    gs_fstatic = p[0];
    u_i(sink(1));
    return gs_fstatic;
}

extern "C" int ga_lstatic(int *p) {
    static int s;
    s = p[0];
    u_i(sink(1));
    return s;
}

// ---- A4/A11/A12, kind 3: the compiler-generated TEMPORARY -----------------
// `f1(x)`'s result is a value with no source name.  It must survive the call
// to `g1(y)`.  If kind 3 were rejected by A11 or A12 it could not be a
// candidate and would have to be homed in the frame across that call.
extern "C" int ga_temp(int x, int y) {
    return f1(x) + g1(y);
}

// Two temporaries live across two calls.
extern "C" int ga_temp3(int x, int y, int z) {
    return f1(x) + g1(y) + h1(z);
}

// ---- A3 + A10: the sub-symbol chain, and the `t+0x20 == 4` width test -----
// A LOCAL aggregate is a kind-4/5 symbol (linkage 1/3), so it takes the
// general path at 0x10b551ca — which gate-B's each sub-symbol INDIVIDUALLY and
// applies NO width test.  A10's `t+0x20 == 4` belongs to the kind-10 arm only.
// Prediction: every member promotes, whatever its width.
struct Mix { int a; char b; short c; long long d; };

extern "C" long long ga_structmix(Mix *p) {
    Mix v;
    v.a = p->a;
    v.b = p->b;
    v.c = p->c;
    v.d = p->d;
    u_i(sink(1));
    return (long long)v.a + v.b + v.c + v.d;
}

// The same shape with every member 4 bytes wide — the control for the width
// test.  If ga_structmix and ga_struct4 differ ONLY on the non-4-byte members,
// the width test is live for local aggregates too and the prediction is wrong.
struct Four { int a, b, c, d; };

extern "C" int ga_struct4(Four *p) {
    Four v;
    v.a = p->a;
    v.b = p->b;
    v.c = p->c;
    v.d = p->d;
    u_i(sink(1));
    return v.a + v.b + v.c + v.d;
}

// ---- A9, kind 0xb: a FUNCTION symbol -------------------------------------
// `&f1` is a kind-0xb symbol (gl kind 3).  The POINTER is an auto (kind 4/5).
extern "C" int ga_fnaddr(int x) {
    int (*fp)(int) = f1;
    u_i(sink(1));
    return fp(x);
}

// ---- A6 again, through a formal and through a reference ------------------
extern "C" int ga_param(int x) {
    u_i(sink(1));
    return x;
}

extern "C" int ga_ref(int &r) {
    int x = r;
    u_i(sink(1));
    return x;
}

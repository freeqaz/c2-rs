// promote_grid.cpp — lane w-globobj (L3 of docs/ADOPTION_BRIEF_2026-08-28.md).
//
// THE FIRST OBJ CELLS FOR THE PROMOTION POLICY, docs/whitebox/ref/P_GLOBREGS.md §3.
//
// §3 is the answer to a question three documents call uncharacterized
// (`WB_LIVE_FINDINGS.md:682`, `WB_ITEMF_FINDINGS.md` F1, `P_REGALLOC.md` §7):
// which symbols become register-allocation candidates.  It is read out of
// `FUN_10b550e5` as a structural gate A plus a categorical type gate B
// (`FUN_10bd7d24`, the byte at `0x10b18b28 + class*4`), with **no threshold
// constant anywhere**.  Every word of that is `[R]`.  These are the cells.
//
// THE READOUT — the frame-traffic rule, and it is uniform across every type in
// this file, which is why it was chosen:
//
//     A promoted local needs no stack slot.  A rejected one is homed in the
//     frame.  So: disassemble the cell, find the `stwu r1, -N(r1)` that opens
//     the frame and the `addi r1, r1, N` that closes it, and ask whether any
//     STORE between them targets an `r1`-relative slot (or a relocated static
//     address).  The prologue's own `stw r12,-8(r1)` / `std r31,-16(r1)`
//     register saves sit BEFORE the `stwu` and are excluded by construction,
//     not by a heuristic.
//
// Predictions frozen in work/w-globobj/PREREG.md §4 BEFORE this file was
// compiled.  `pc_int` is the positive control (must be PROMOTED); `pc_vol` is
// the negative control (must be MEMORY).  `pc_struct1` is registered OPEN — no
// prediction — because a one-word aggregate is exactly where "aggregate ⇒
// reject" could break and a lane that predicts both ways predicts nothing.
//
// Every cell has the same shape: load the local from an indexed pointer
// formal, cross a call, use it.  The call is what forces the question — a
// value that does not survive a call needs no callee-saved register and no
// slot, so a body without one grades nothing.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/promote_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W, the workload profile)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/promote_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X, the fixture profile)
// Grade:    docs/whitebox/scripts/grade_globobj.py --promote <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);
extern "C" void u_ll(long long);
extern "C" void u_p(void *);
extern "C" void u_f(float);
extern "C" void u_d(double);
extern "C" void esc(int *);

enum E { E0, E1, E2 };
struct S2 { int a, b; };
struct S1 { int a; };
union U { int a; float b; };

// ---- predicted PROMOTED -----------------------------------------------------

extern "C" int pc_int(int *p)
{
    int v = p[0];
    int t = sink(7);
    u_i(v);
    return t;
}

extern "C" int pc_uchar(unsigned char *p)
{
    unsigned char v = p[0];
    int t = sink(7);
    u_i(v);
    return t;
}

extern "C" int pc_short(short *p)
{
    short v = p[0];
    int t = sink(7);
    u_i(v);
    return t;
}

extern "C" int pc_ll(long long *p)
{
    long long v = p[0];
    int t = sink(7);
    u_ll(v);
    return t;
}

extern "C" int pc_ptr(void **p)
{
    void *v = p[0];
    int t = sink(7);
    u_p(v);
    return t;
}

extern "C" int pc_bool(bool *p)
{
    bool v = p[0];
    int t = sink(7);
    u_i(v ? 1 : 0);
    return t;
}

extern "C" int pc_enum(E *p)
{
    E v = p[0];
    int t = sink(7);
    u_i((int)v);
    return t;
}

extern "C" int pc_float(float *p)
{
    float v = p[0];
    int t = sink(7);
    u_f(v);
    return t;
}

extern "C" int pc_double(double *p)
{
    double v = p[0];
    int t = sink(7);
    u_d(v);
    return t;
}

// ---- predicted MEMORY -------------------------------------------------------

extern "C" int pc_struct2(S2 *p)
{
    S2 v = *p;
    int t = sink(7);
    u_i(v.a);
    u_i(v.b);
    return t;
}

// ---- OPEN, no prediction ----------------------------------------------------

extern "C" int pc_struct1(S1 *p)
{
    S1 v = *p;
    int t = sink(7);
    u_i(v.a);
    return t;
}

// ---- predicted MEMORY -------------------------------------------------------

extern "C" int pc_arr(int *p)
{
    int v[2];
    v[0] = p[0];
    v[1] = p[1];
    int t = sink(7);
    u_i(v[0]);
    u_i(v[1]);
    return t;
}

extern "C" int pc_union(U *p)
{
    U v = *p;
    int t = sink(7);
    u_i(v.a);
    return t;
}

extern "C" int pc_addr(int *p)
{
    int v = p[0];
    int t = sink(7);
    esc(&v);
    u_i(v);
    return t;
}

extern "C" int pc_vol(int *p)
{
    volatile int v = p[0];
    int t = sink(7);
    u_i(v);
    return t;
}

extern "C" int pc_static(int *p)
{
    static int v;
    v = p[0];
    int t = sink(7);
    u_i(v);
    return t;
}

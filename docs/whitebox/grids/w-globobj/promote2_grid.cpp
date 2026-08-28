// promote2_grid.cpp — lane w-globobj, prereg addendum 1 §2.
//
// THE CONFOUND `promote_grid.cpp` OPENED, and the cells that decide it.
//
// `promote_grid.cpp` came back with `pc_arr` (`int[2]`) and `pc_union`
// PROMOTED and `pc_struct2` MEMORY.  The obvious reading — "aggregates are
// rejected" — is dead.  The reading that replaces it has a confound that has to
// be named before it is tested:
//
//     `pc_struct2` writes `S2 v = *p;`, which the FRONT END lowers to a single
//     8-byte whole-object copy (`ld 11, 0(3)` / `std 11, 80(1)` in the obj).
//     `pc_arr` writes `v[0] = p[0]; v[1] = p[1];` — two scalar assignments.
//     The difference may be entirely c1xx's and carry no information about
//     `FUN_10b550e5`'s gate A or gate B at all.
//
// `pa_struct2mem` is the same TYPE as `pc_struct2` assigned MEMBER-WISE.  If it
// is PROMOTED, `pc_struct2`'s MEMORY verdict is a front-end artifact and this
// lane will say so rather than bank it as a gate-B confirmation.
//
// THE ARRAY LADDER tests `P_GLOBREGS.md` §3's headline negative — "no size
// threshold, no use-count threshold ... a port therefore needs no fitted
// constant for F1" — at the observable.
//
// THE CEILING ON THE LADDER, registered in the addendum before this file was
// compiled: the frame-traffic readout CANNOT separate "was never promoted" from
// "was promoted and then spilled".  Above roughly a dozen simultaneously live
// values the callee-saved run `r14…r31` runs out and spilling is the correct
// behaviour of a working allocator.  `pa_arr12` is reported as DATA, not as a
// graded cell, and no threshold claim is made from it in either direction.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/promote2_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/promote2_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X)
// Grade:    docs/whitebox/scripts/grade_globobj.py --promote <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

struct S2 { int a, b; };
struct S4 { int a, b, c, d; };

// ---- the control: the pc_struct2 shape, restated here ----------------------

extern "C" int pa_struct2cpy(S2 *p)
{
    S2 v = *p;
    int t = sink(7);
    u_i(v.a);
    u_i(v.b);
    return t;
}

// ---- the same TYPE, assigned member-wise -----------------------------------

extern "C" int pa_struct2mem(S2 *p)
{
    S2 v;
    v.a = p->a;
    v.b = p->b;
    int t = sink(7);
    u_i(v.a);
    u_i(v.b);
    return t;
}

extern "C" int pa_struct4mem(S4 *p)
{
    S4 v;
    v.a = p->a;
    v.b = p->b;
    v.c = p->c;
    v.d = p->d;
    int t = sink(7);
    u_i(v.a);
    u_i(v.b);
    u_i(v.c);
    u_i(v.d);
    return t;
}

// ---- the array ladder ------------------------------------------------------

extern "C" int pa_arr4(int *p)
{
    int v[4];
    for (int i = 0; i < 4; i++) v[i] = p[i];
    int t = sink(7);
    u_i(v[0]); u_i(v[1]); u_i(v[2]); u_i(v[3]);
    return t;
}

extern "C" int pa_arr4u(int *p)
{
    int v[4];
    v[0] = p[0]; v[1] = p[1]; v[2] = p[2]; v[3] = p[3];
    int t = sink(7);
    u_i(v[0]); u_i(v[1]); u_i(v[2]); u_i(v[3]);
    return t;
}

extern "C" int pa_arr8(int *p)
{
    int v[8];
    v[0] = p[0]; v[1] = p[1]; v[2] = p[2]; v[3] = p[3];
    v[4] = p[4]; v[5] = p[5]; v[6] = p[6]; v[7] = p[7];
    int t = sink(7);
    u_i(v[0]); u_i(v[1]); u_i(v[2]); u_i(v[3]);
    u_i(v[4]); u_i(v[5]); u_i(v[6]); u_i(v[7]);
    return t;
}

// ---- DATA ONLY, not a graded cell: above the readout's soundness -----------

extern "C" int pa_arr12(int *p)
{
    int v[12];
    v[0] = p[0]; v[1] = p[1]; v[2] = p[2];  v[3] = p[3];
    v[4] = p[4]; v[5] = p[5]; v[6] = p[6];  v[7] = p[7];
    v[8] = p[8]; v[9] = p[9]; v[10] = p[10]; v[11] = p[11];
    int t = sink(7);
    u_i(v[0]); u_i(v[1]); u_i(v[2]);  u_i(v[3]);
    u_i(v[4]); u_i(v[5]); u_i(v[6]);  u_i(v[7]);
    u_i(v[8]); u_i(v[9]); u_i(v[10]); u_i(v[11]);
    return t;
}

// order_grid.cpp — lane w-globobj (L3 of docs/ADOPTION_BRIEF_2026-08-28.md).
//
// THE SEPARATOR `docs/whitebox/ref/P_GLOBREGS.md` §7.1 says was never built.
//
// §7.1 registers a cell (`scripts/globregs_c2.py`'s G-block) that came back
// UNGRADED, and closes: "the separator remains unbuilt".  The reason it was
// unbuilt is that every previous grid varied the USE order or the BLOCK
// position, both of which move `cand+0x0c` — the comparator's PRIMARY key —
// so the tie tier was never reached, and both of which leave the *declaration*
// order and the *definition* order welded together.
//
// THE ENTAILMENT THIS GRID TESTS, which nothing on the page states.  Compose
// §7.1's two directions:  the step-4 walk is blocks FORWARD x tuples BACKWARD
// (`0x10b55eb4`/`0x10b55ebc` vs `T->[0x10]`), the counter is not reset per
// block (`0x10b55eb7` is outside the block loop), and `cand+0x44` is
// overwritten at every encounter (`0x10b55fac`) so the surviving value is the
// LAST visit.  In a single-block straight-line body the tuples are visited
// last-to-first, so a candidate's last visit is its EARLIEST tuple in program
// order — which for a local is its DEFINITION.  `0x10b2b82d` sorts `+0x44`
// DESC.  Therefore:
//
//     the EARLIEST-DEFINED candidate is coloured first.
//
// That is a source-observable rule, and it disagrees with plain arena order
// exactly when the definition order and the declaration order disagree — which
// costs one line of C and which no grid in this repo had ever written.
//
// THE THREE AXES, independent by construction:
//
//     declaration order   `int x, y;`   vs   `int y, x;`
//     definition  order   `x=p[0]; y=p[1];`  vs  `y=p[1]; x=p[0];`
//     use         order   `u_i(x); u_i(y);`  vs  `u_i(y); u_i(x);`
//
// THE READOUT.  Both locals are defined by an indexed load off ONE pointer
// formal, so `p[0]` and `p[1]` are told apart in the obj by their
// DISPLACEMENT, and the destination register of each `lwz` is that local's
// colour.  This deliberately avoids R4's formal->register readout
// (`scripts/globregs_c2.py`), whose formals arrive in ABI registers — an
// arrival register is itself a declaration-side property and confounds every
// declaration-order rival.
//
// THE CEILING, registered in work/w-globobj/PREREG.md §5.4 BEFORE this file was
// compiled: moving a definition earlier both raises `+0x44` and lengthens the
// live interval, hence `+0x0c` (`P_REGALLOC.md`:71, `cand[0x0c] += cand[0x18]
// * n_live`).  This grid therefore grades the COMPOSITE order.  It cannot
// attribute the result to `+0x44` rather than `+0x0c`, and no verdict here
// claims to.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W, the workload profile)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X, the fixture profile)
// Grade:    docs/whitebox/scripts/grade_globobj.py --order <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

// ---- N=2: 2 declaration orders x 2 definition orders x 2 use orders --------
// Cell name is oc2_<decl>_<def>_<use>.

extern "C" int oc2_xy_xy_xy(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y);
    return t;
}

extern "C" int oc2_xy_xy_yx(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(y); u_i(x);
    return t;
}

extern "C" int oc2_xy_yx_xy(int *p)
{
    int x, y;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(x); u_i(y);
    return t;
}

extern "C" int oc2_xy_yx_yx(int *p)
{
    int x, y;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(y); u_i(x);
    return t;
}

extern "C" int oc2_yx_xy_xy(int *p)
{
    int y, x;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y);
    return t;
}

extern "C" int oc2_yx_xy_yx(int *p)
{
    int y, x;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(y); u_i(x);
    return t;
}

extern "C" int oc2_yx_yx_xy(int *p)
{
    int y, x;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(x); u_i(y);
    return t;
}

extern "C" int oc2_yx_yx_yx(int *p)
{
    int y, x;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(y); u_i(x);
    return t;
}

// ---- N=3: declaration order FIXED (x,y,z), all 6 definition orders, uses
// ---- FIXED (x,y,z).  DECL predicts all six identical; DEF predicts the map
// ---- follows the permutation.  Cell name is oc3_<def permutation>.

extern "C" int oc3_xyz(int *p)
{
    int x, y, z;
    x = p[0]; y = p[1]; z = p[2];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

extern "C" int oc3_xzy(int *p)
{
    int x, y, z;
    x = p[0]; z = p[2]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

extern "C" int oc3_yxz(int *p)
{
    int x, y, z;
    y = p[1]; x = p[0]; z = p[2];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

extern "C" int oc3_yzx(int *p)
{
    int x, y, z;
    y = p[1]; z = p[2]; x = p[0];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

extern "C" int oc3_zxy(int *p)
{
    int x, y, z;
    z = p[2]; x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

extern "C" int oc3_zyx(int *p)
{
    int x, y, z;
    z = p[2]; y = p[1]; x = p[0];
    int t = sink(7);
    u_i(x); u_i(y); u_i(z);
    return t;
}

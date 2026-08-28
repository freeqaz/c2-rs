// version_grid.cpp — lane w-globobj (L3 of docs/ADOPTION_BRIEF_2026-08-28.md).
//
// "A VARIABLE IS NOT A CANDIDATE" — `docs/whitebox/ref/P_GLOBREGS.md` §1 step 3
// and §8 consequence 3.
//
// Step 3 (`FUN_10b55dbe`, `0x10b55e5d`/`0x10b55e66`) mints one candidate per
// VERSION RECORD, not per symbol: a symbol with *k* versions mints *k*
// candidates, each with its own `+0x44`.  §8 leans the whole explanation of
// `codegen/alloc.rs`'s ten refuted allocation keys on that sentence — every one
// of the ten is one-candidate-per-variable by construction and is therefore
// "wrong in kind, not merely mis-fitted, on any body where a value is
// redefined".  The sentence is `[R]`.
//
// THE OBSERVABLE, and why one cell is not enough.  "The two ranges got
// different registers" alone carries no information — any allocator may reuse a
// register.  The cell that carries information is the PAIR: a redefined
// variable against two distinct variables in the same shape.  If the version
// model is right the two are behaviourally indistinguishable in the obj.
//
//     vc_reuse     x = p[0]; call; use x; x = p[1]; call; use x
//     vc_distinct  x = p[0]; call; use x; z = p[1]; call; use z
//     vc_single    x = p[0]; call; call; use x                     (control)
//
// PREDICTION, frozen in work/w-globobj/PREREG.md §6 before this file was
// compiled: `vc_reuse` and `vc_distinct` produce the SAME register map, and
// `vc_single` uses ONE register.  The refuting outcome is `vc_reuse` pinning
// both ranges to one register while `vc_distinct` uses two — that would refute
// the *k*-versions claim at the observable and take §8 consequence 3's support
// away with it.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/version_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W, the workload profile)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/version_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X, the fixture profile)
// Grade:    docs/whitebox/scripts/grade_globobj.py --version <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

extern "C" int vc_reuse(int *p)
{
    int x;
    x = p[0];
    int t = sink(7);
    u_i(x);
    x = p[1];
    int t2 = sink(8);
    u_i(x);
    return t + t2;
}

extern "C" int vc_distinct(int *p)
{
    int x, z;
    x = p[0];
    int t = sink(7);
    u_i(x);
    z = p[1];
    int t2 = sink(8);
    u_i(z);
    return t + t2;
}

extern "C" int vc_single(int *p)
{
    int x;
    x = p[0];
    int t = sink(7);
    int t2 = sink(8);
    u_i(x);
    return t + t2;
}

// ---- vc_three: three versions of one symbol.  If the mint is per version the
// ---- body behaves like three distinct locals; if it is per symbol the
// ---- allocator has one candidate to place and the shape must differ.

extern "C" int vc_three(int *p)
{
    int x;
    x = p[0];
    int t = sink(7);
    u_i(x);
    x = p[1];
    int t2 = sink(8);
    u_i(x);
    x = p[2];
    int t3 = sink(9);
    u_i(x);
    return t + t2 + t3;
}

extern "C" int vc_three_distinct(int *p)
{
    int x, y, z;
    x = p[0];
    int t = sink(7);
    u_i(x);
    y = p[1];
    int t2 = sink(8);
    u_i(y);
    z = p[2];
    int t3 = sink(9);
    u_i(z);
    return t + t2 + t3;
}

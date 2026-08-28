// order_lr_grid.cpp — lane w-globobj, prereg addendum 1 §3.
//
// THE CELL THAT ATTACKS THIS LANE'S OWN REGISTERED CEILING.
//
// `work/w-globobj/PREREG.md` §5.4 registered, before any cell was compiled,
// that this lane could not separate `cand+0x44` from `cand+0x0c`: moving a
// definition earlier both raises the tuple-visit ordinal and lengthens the live
// interval, and `P_REGALLOC.md`:71 reads the priority accumulator as
// `cand[0x0c] += cand[0x18] * n_live` — a function of that interval.
//
// **That ceiling is attackable and the prereg did not see how.** Hold the
// DEFINITION order fixed and move the LAST USE: the live interval changes and
// the definition ordinal does not.  Then make the LATER-defined candidate the
// LONGER-lived one, and the two rivals point opposite ways.
//
//     ol_dxy_ylate   x defined first, y lives longest   <- DISCRIMINATOR
//     ol_dyx_xlate   y defined first, x lives longest   <- DISCRIMINATOR
//     ol_dxy_xlate   x defined first, x lives longest   (consistency)
//     ol_dyx_ylate   y defined first, y lives longest   (consistency)
//
// | rival    | ol_dxy_ylate | ol_dyx_xlate |
// |----------|--------------|--------------|
// | DEF      | x -> r31     | y -> r31     |
// | LIVELEN  | y -> r31     | x -> r31     |
//
// The padding is three `t = sink(t)` calls — real tuples that touch neither
// local, so they lengthen one interval without adding a candidate that could
// itself perturb `n_live` asymmetrically.
//
// The `ou_` pair does the same on the USE-COUNT axis, because `cand+0x18` is a
// per-candidate weight and use count is the obvious thing for it to be.
//
// WHAT A DEF WIN LICENSES, registered in the addendum before this file was
// compiled: NOT `[O]` on `cand+0x44` — the obj shows a composite order, and a
// `+0x0c` that is itself ordered by definition position would look the same.
// It licenses the narrower and true statement that **live-range extent and use
// count do not order the observable**, which narrows §5.4's residue instead of
// closing it.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_lr_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_lr_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X)
// Grade:    docs/whitebox/scripts/grade_globobj.py --order <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

// ---- the live-range axis ---------------------------------------------------

extern "C" int ol_dxy_xlate(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(y);
    t = sink(t); t = sink(t); t = sink(t);
    u_i(x);
    return t;
}

extern "C" int ol_dxy_ylate(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x);
    t = sink(t); t = sink(t); t = sink(t);
    u_i(y);
    return t;
}

extern "C" int ol_dyx_xlate(int *p)
{
    int x, y;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(y);
    t = sink(t); t = sink(t); t = sink(t);
    u_i(x);
    return t;
}

extern "C" int ol_dyx_ylate(int *p)
{
    int x, y;
    y = p[1]; x = p[0];
    int t = sink(7);
    u_i(x);
    t = sink(t); t = sink(t); t = sink(t);
    u_i(y);
    return t;
}

// ---- the use-count axis ----------------------------------------------------

extern "C" int ou_x2(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(x); u_i(y);
    return t;
}

extern "C" int ou_y2(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y); u_i(y);
    return t;
}

extern "C" int ou_y3(int *p)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x); u_i(y); u_i(y); u_i(y);
    return t;
}

// order_loop_grid.cpp — lane w-globobj, prereg addendum 2 §C.
//
// THE CELL THAT CAN BREAK THIS LANE'S REGISTERED CEILING.
//
// After 42 graded cells, `DEF` — the earliest-DEFINED candidate is coloured
// first — is unbeaten and `LIVELEN` is refuted by 12.  `P_REGALLOC.md`:71 reads
// the priority accumulator as `cand[0x0c] += cand[0x18] * n_live` where live,
// `-= n_live` where not, and under EITHER sign of `cand+0x18` that formula is a
// monotone function of live extent and therefore predicts `LIVELEN`.  Two
// survivors are left:
//
//   1. `+0x0c` TIES on those cells and the comparator falls to `+0x44` — which
//      is `P_GLOBREGS.md` §7.1's ordinal reading, at the observable; or
//   2. `+0x0c` is itself ordered by definition position and `P_REGALLOC`:71 is
//      wrong or incomplete.
//
// These cells separate them.  If `+0x0c` can be made to move by a source-level
// quantity that is NOT definition position, survivor 2 is dead and the 42
// straight-line cells were decided in the TIE TIER.  The quantity is a
// loop-weighted use count: `cand+0x18` is a per-candidate weight and loop depth
// is what a priority accumulator exists to express.
//
//     ob_loop_y   x defined first, y used inside a loop      <- DISCRIMINATOR
//     ob_loop_x   x defined first, x used inside a loop      (consistency)
//     ob_loop2_y  x defined first, y used at loop depth 2    (deeper)
//
// | rival       | ob_loop_y |
// |-------------|-----------|
// | DEF         | x -> r31  |
// | LOOPWEIGHT  | y -> r31  |
//
// PREDICTION, frozen in the addendum before this file was compiled: y -> r31.
// If x -> r31 instead, the ceiling registered in PREREG.md §5.4 is CONFIRMED by
// a cell built to break it, and this lane says so.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_loop_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/order_loop_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X)
// Grade:    docs/whitebox/scripts/grade_globobj.py --order <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

extern "C" int ob_loop_y(int *p, int n)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x);
    for (int i = 0; i < n; i++) u_i(y);
    return t;
}

extern "C" int ob_loop_x(int *p, int n)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    for (int i = 0; i < n; i++) u_i(x);
    u_i(y);
    return t;
}

extern "C" int ob_loop2_y(int *p, int n)
{
    int x, y;
    x = p[0]; y = p[1];
    int t = sink(7);
    u_i(x);
    for (int j = 0; j < n; j++)
        for (int i = 0; i < n; i++) u_i(y);
    return t;
}

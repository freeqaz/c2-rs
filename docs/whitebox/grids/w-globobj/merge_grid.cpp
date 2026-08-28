// merge_grid.cpp — lane w-globobj, prereg addendum 1 §4.
//
// THE MERGE AT JOINS — `docs/whitebox/ref/P_GLOBREGS.md` §5, `FUN_10b54c07`
// at `0x10b54c07`.
//
// `work/w-globobj/MARKS.tsv` filed line 169 as **UNCOMP** — a cell could exist,
// this lane does not build it — and named the cell.  It costs three lines, so
// the lane builds it rather than leaving a question a later lane has to
// re-derive.  `MARKS.tsv`'s standing rule is that UNCOMP is a statement about
// what this lane did, never about what the corpus contains.
//
// §5 reads the merge as **keyed on the symbol**, reached through
// `DAT_10c400d0[i]` — *"two definitions of the same symbol merge, two different
// symbols never do"* — and as either REUSING an existing version number whose
// bitset already meets the join's phi set, or minting a fresh one.
//
//     vm_merge    one symbol, two definitions, one join, one use after it
//     vm_nomerge  two symbols, each confined to its own arm — nothing to merge
//     vm_merge3   one symbol, three definitions reaching one join
//
// PREDICTION frozen in the addendum before this file was compiled: **both arms
// of `vm_merge` load into the SAME register** — one candidate survives the
// join.  Different registers plus a copy at the join would say the merge minted
// a fresh version per arm.  Either result is reported as data; neither is a
// pass/fail, because §5 admits both behaviours and the cell's job is to say
// which one this shape takes.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globobj/merge_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globobj/merge_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X)
// Grade:    docs/whitebox/scripts/grade_globobj.py --merge <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);

extern "C" int vm_merge(int *p, int c)
{
    int x;
    if (c) x = p[0];
    else   x = p[1];
    int t = sink(7);
    u_i(x);
    return t;
}

extern "C" int vm_nomerge(int *p, int c)
{
    int x, y;
    if (c) { x = p[0]; int t = sink(7); u_i(x); return t; }
    y = p[1];
    int t = sink(8);
    u_i(y);
    return t;
}

extern "C" int vm_merge3(int *p, int c)
{
    int x;
    if (c == 1)      x = p[0];
    else if (c == 2) x = p[1];
    else             x = p[2];
    int t = sink(7);
    u_i(x);
    return t;
}

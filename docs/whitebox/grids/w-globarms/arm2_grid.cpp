// arm2_grid.cpp — lane w-globarms, addendum 1.
//
// A6's INTERNAL test, which is the one separator on this page that does not
// leave the arm.
//
// `P_GLOBREGS.md` §3 gate A row `0x10b5513e`:
//
//     kind in {4,5}  ->  eligible; `sym+0x05 & 2` set => ALSO joins the
//                        DAT_10c2e3e8 set
//
// and row `0x10b5514a`: kinds 7,8 join that set ALWAYS.  So A6 is the only arm
// with a per-symbol branch that source can move without changing the kind:
// the same declaration, the same type, the same TU, the same optimisation
// profile — only whether `&x` reaches an opaque callee.
//
// `FUN_10bd2db7` (`0x10bd2db7`) is the ONLY thing that sets `+0x05 |= 2`, and
// it walks the leader's `+0x0c` sub-symbol chain, so the flag is a property of
// a SYMBOL GROUP, not of one member.
//
// `gb_pair_yescape` is the deciding cell: two locals of the same type in one
// body, one of which has its address escape.  If the flag is per-symbol, x
// stays in a register while y is homed.  If it is per-function, both go to
// memory.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-globarms/arm2_grid.cpp \
//               /nologo /Gy /O1 /GS- /c        (mode W)
//           scripts/gt_capture.sh docs/whitebox/grids/w-globarms/arm2_grid.cpp \
//               /nologo /Gy /Ox /GS- /c        (mode X)
// Grade:    docs/whitebox/scripts/grade_globarms.py --arms <dump.txt> ...

extern "C" int sink(int);
extern "C" void u_i(int);
extern "C" void u_p(int *);
extern "C" int f1(int);

// The address is taken but never leaves the function.
extern "C" int gb_addr_local(int *p) {
    int x = p[0];
    int *q = &x;
    u_i(sink(1));
    return *q;
}

// The address escapes into an opaque callee.
extern "C" int gb_addr_escape(int *p) {
    int x = p[0];
    u_p(&x);
    u_i(sink(1));
    return x;
}

// THE DECIDING CELL — two locals, one escapes.
extern "C" int gb_pair_yescape(int *p) {
    int x = p[0];
    int y = p[1];
    u_p(&y);
    u_i(sink(1));
    return x + y;
}

// The mirror: the OTHER one escapes.
extern "C" int gb_pair_xescape(int *p) {
    int x = p[0];
    int y = p[1];
    u_p(&x);
    u_i(sink(1));
    return x + y;
}

// Neither escapes — the control for the pair.
extern "C" int gb_pair_none(int *p) {
    int x = p[0];
    int y = p[1];
    u_i(sink(1));
    return x + y;
}

// A9 / kind 0xb: the function symbol materialised as an address and used
// twice, so a candidate would pay off if one existed.
extern "C" int gb_fnaddr2(int a, int b) {
    int (*fp)(int) = f1;
    return fp(a) + fp(b);
}

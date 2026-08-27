// physdef_grid.cpp — lane w-regcells, question Q2.
//
// F4's NON-CALL PHYSICAL DEF — the falsifier battery.
//
// `P_REGALLOC.md` §7: "F4's non-call physical def: still no obj cell in
// existence."  `WB_LIVE_FINDINGS.md` §6.2 records the mechanism as
// disassembly-only: `FUN_10b2d630` clears a physically-defined register from
// `cand->allowed` for every candidate on the live list, and the reading says
// this happens for a BARE physical def, not only for a call tuple's kind-0x0b
// clobber-set operand.
//
// work/w-regcells/PREREG.md §2.2 registers a NEGATIVE prediction — that no C
// source shape on this target satisfies all three of (a) a non-call physical
// def of an ALLOCATABLE GPR, (b) a candidate live across it, (c) an observable
// displacement — because the front end has exactly three sources of an
// allocatable-GPR physical def (formal arrival, call-sequence argument setup,
// return-value materialisation) and each fails a different one.
//
// EVERY CELL BELOW IS AN ATTEMPT TO FALSIFY THAT.  None is a demonstration of
// it.  Each `_p` twin adds nine simultaneously-live integer globals: PREREG
// §2.4's hardwired-register test is that a genuinely hardwired register keeps
// its role in both twins while the surrounding allocation demonstrably moves.
//
// Compile:  scripts/gt_capture.sh docs/whitebox/grids/w-regcells/physdef_grid.cpp \
//               /nologo /c /GR /O1 /Oi /EHsc
// Read:     scripts/gt_dump.py <obj>

extern "C" {

extern int qg0, qg1, qg2, qg3, qg4, qg5, qg6, qg7, qg8, qg9;
extern int gg2(int, int);
extern void g8(int, int, int, int, int, int, int, int);

#define PRESS_DECL                                                            \
    int p0 = qg0, p1 = qg1, p2 = qg2, p3 = qg3, p4 = qg4,                     \
        p5 = qg5, p6 = qg6, p7 = qg7, p8 = qg8
#define PRESS_USE (((p0 * p1) + (p2 * p3)) + ((p4 * p5) + (p6 * p7)) + p8)

// ---- pd_ctr: a dense switch.  Predicted `mtctr`/`bctr` — and `ctr` is
// ---- register 84 on the 0x10b181c0 table, which appears in NO class list
// ---- (0x10c385c4 has only classes 0 and 1), so a physical def of it should
// ---- clear a bit that is in no GPR candidate's set and displace nothing.
// ---- This is the battery's NEGATIVE CONTROL and it is the arm most likely
// ---- to go red if the reading of the class map is wrong.

int pd_ctr(int k)
{
    switch (k) {
    case 0: return 11;  case 1: return 22;  case 2: return 33;
    case 3: return 44;  case 4: return 55;  case 5: return 66;
    case 6: return 77;  case 7: return 88;
    }
    return 99;
}
int pd_ctr_p(int k)
{
    PRESS_DECL;
    int r;
    switch (k) {
    case 0: r = 11; break;  case 1: r = 22; break;  case 2: r = 33; break;
    case 3: r = 44; break;  case 4: r = 55; break;  case 5: r = 66; break;
    case 6: r = 77; break;  case 7: r = 88; break;
    default: r = 99; break;
    }
    return r + PRESS_USE;
}

// ---- pd_tail: a PERMUTED TAIL call.  `b gg2` rather than `bl gg2`: the
// ---- argument registers r3/r4 are physically defined and there is no `bl` in
// ---- the body at all.  If a bare physical def narrows anything observably,
// ---- the scratch that breaks the 2-cycle is where it would show.

int pd_tail(int a, int b) { return gg2(b, a); }
int pd_tail_p(int a, int b)
{
    PRESS_DECL;
    return gg2(b + PRESS_USE, a);
}

// ---- pd_argdie: ten values from globals; EIGHT go to a call, and the last
// ---- two die BEFORE the `bl`.  A value that avoids r3..r10 while provably
// ---- not live across the call would be the cell F4 has never had.

int pd_argdie(void)
{
    int v0 = qg0, v1 = qg1, v2 = qg2, v3 = qg3, v4 = qg4;
    int v5 = qg5, v6 = qg6, v7 = qg7, v8 = qg8, v9 = qg9;
    int dies = (v8 * v9) + (v8 ^ v9);
    g8(v0 + dies, v1, v2, v3, v4, v5, v6, v7);
    return 0;
}
int pd_argdie_p(void)
{
    PRESS_DECL;
    int v0 = qg0, v1 = qg1, v2 = qg2, v3 = qg3, v4 = qg4;
    int v5 = qg5, v6 = qg6, v7 = qg7, v8 = qg8, v9 = qg9;
    int dies = (v8 * v9) + (v8 ^ v9);
    g8(v0 + dies + PRESS_USE, v1, v2, v3, v4, v5, v6, v7);
    return 0;
}

// ---- pd_ret2: two `return`s, so `r3` is materialised on one path while the
// ---- other path's values are still around in the CFG.  Tests condition (b):
// ---- is anything live across a return-value def?

int pd_ret2(const int *a, int n)
{
    int s = 0;
    for (int i = 0; i < n; ++i) {
        s += a[i];
        if (s > 100) return 7;
    }
    return s;
}
int pd_ret2_p(const int *a, int n)
{
    PRESS_DECL;
    int s = 0;
    for (int i = 0; i < n; ++i) {
        s += a[i];
        if (s > 100) return 7 + PRESS_USE;
    }
    return s + PRESS_USE;
}

// ---- pd_lr: the frame's `mflr r12` shuttle.  r12 is register 13 on the name
// ---- table and is ABSENT from 0x10c37de0's 27 entries, so this is the second
// ---- negative control: a hardwired physical def of a NON-allocatable
// ---- register, with a live candidate straddling it, must displace nothing.

extern void gsink(void);
int pd_lr(void)
{
    PRESS_DECL;
    gsink();
    return PRESS_USE;
}


// ---- pd_perm6 / pd_perm8: THE CELLS ADDENDUM 1 REGISTERS.
// ---- `--pure` argument permutations (docs/CODEGEN_ARG_PERM.md §2, §5): a TAIL
// ---- call, so the body is nothing but `mr`s and a `b` — NO `bl`, no call
// ---- tuple, no kind-0x0b clobber operand anywhere — and every `mr` into an
// ---- argument register is a BARE PHYSICAL DEF with candidates live across it.
// ----
// ---- pd_perm6 is the POSITIVE CONTROL: CODEGEN_ARG_PERM §5.1 pins the three
// ---- scratches to r11, r10, r9 on 61 of 61 three-minima cells. If this lane's
// ---- capture does not reproduce that verbatim, pd_perm8 decides nothing.
// ----
// ---- pd_perm8 is the new one: sigma = (r3 r10)(r4 r9)(r5 r8)(r6 r7), four
// ---- 2-cycles, four local minima -> four scratches, and the eight formals
// ---- occupy r3..r10 leaving exactly ONE free volatile. PREREG addendum 1
// ---- §A1.2 P-B predicts r11, r31, r30, r29 — class 0's list order with
// ---- r3..r10 removed — and NOT r14/r15/r16, and NOT r12/r0/r13.

extern void gg6(int, int, int, int, int, int);
extern void gg8(int, int, int, int, int, int, int, int);

void pd_perm6(int a, int b, int c, int d, int e, int f)
{
    gg6(d, e, f, a, b, c);          // §5.1's exact cell: (r3 r6)(r4 r7)(r5 r8)
}

void pd_perm8(int a, int b, int c, int d, int e, int f, int g, int h)
{
    gg8(h, g, f, e, d, c, b, a);    // (r3 r10)(r4 r9)(r5 r8)(r6 r7)
}

}  // extern "C"

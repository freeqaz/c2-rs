// dagorder_grid.cpp — wb-dagorder's obj grid for the dag.c/scheduler ordering
// reading.
//
// FROZEN BEFORE THE FIRST cl.exe OF THIS LANE, BY CONTENT HASH. Predictions are
// in docs/whitebox/WB_DAGORDER_PREREG_R2.md, committed in the same commit as
// this file.
//
// Compile with the real cl.exe 16.00.11886.00 under wibo at the workload mode
//   /nologo /c /GR /O1 /Oi /EHsc
// reading order from the /FAsc listing AND the obj (gt_dump.py), so no claim
// rests on the listing alone.
//
// The axes this grid varies are STRUCTURAL, per the standing blind spot:
// statement count (1,2,3,4,20), symbol sharing (disjoint/shared source),
// call interposition (none/one, fresh/shared symbol), operand order of a
// non-commutative op (both directions), expression arity (2 vs 3 operands),
// producer kind (loaded value vs literal), and a region-capacity cell (the
// 0x50-tuple cap) plus a guarded cell (compare/branch separation).

extern int dg_a, dg_b, dg_c, dg_d, dg_e, dg_f, dg_g, dg_h;
extern int dg_o0, dg_o1, dg_o2, dg_o3, dg_o4, dg_o5, dg_o6, dg_o7;
extern int dg_i0,  dg_i1,  dg_i2,  dg_i3,  dg_i4,  dg_i5,  dg_i6,  dg_i7,
           dg_i8,  dg_i9,  dg_i10, dg_i11, dg_i12, dg_i13, dg_i14, dg_i15,
           dg_i16, dg_i17, dg_i18, dg_i19;
extern int dg_p0,  dg_p1,  dg_p2,  dg_p3,  dg_p4,  dg_p5,  dg_p6,  dg_p7,
           dg_p8,  dg_p9,  dg_p10, dg_p11, dg_p12, dg_p13, dg_p14, dg_p15,
           dg_p16, dg_p17, dg_p18, dg_p19;

extern "C" void dg_ext(void);

// ONE: the within-statement baseline. Both @ha forms, the load, the add, the
// store of a single statement.
extern "C" void dg_one(void)
{
    dg_o0 = dg_a + 1;
}

// TWO: minimal pair for the phase-grouping question.
extern "C" void dg_two(void)
{
    dg_o0 = dg_a + 1;
    dg_o1 = dg_b + 2;
}

// V1: the wbl_v1 replica — the POSITIVE CONTROL. If the six lis are not at the
// top ahead of the first lwz, the instrument or mode is wrong and no other
// cell is read.
extern "C" void dg_v1(void)
{
    dg_o0 = dg_a + 1;
    dg_o1 = dg_b + 2;
    dg_o2 = dg_c + 3;
}

// V4: the count axis, one past the replica.
extern "C" void dg_v4(void)
{
    dg_o0 = dg_a + 1;
    dg_o1 = dg_b + 2;
    dg_o2 = dg_c + 3;
    dg_o3 = dg_d + 4;
}

// SHARED: two statements reading ONE source symbol. The dedup cell.
extern "C" void dg_shared(void)
{
    dg_o0 = dg_b + 1;
    dg_o1 = dg_b + 2;
}

// DISC: the rank-vs-first-use discriminator (C-DISC). dg_b is read once by the
// FIRST statement; dg_d is read twice by the LATER two statements. First-use
// order puts dg_b's lis first; fanout/use-count rank puts dg_d's first.
extern "C" void dg_disc(void)
{
    dg_o0 = dg_b + 1;
    dg_o1 = dg_d + 2;
    dg_o2 = dg_d + 3;
}

// CALL: the region-boundary control (C-CALL), fresh symbols on each side.
extern "C" void dg_call(void)
{
    dg_o0 = dg_a + 1;
    dg_ext();
    dg_o1 = dg_b + 2;
}

// CALL2: the SAME source symbol on both sides of the call. Rematerialization.
extern "C" void dg_call2(void)
{
    dg_o0 = dg_b + 1;
    dg_ext();
    dg_o1 = dg_b + 2;
}

// SUB / SUB2: a non-commutative operator, both operand orders.
extern "C" void dg_sub(void)
{
    dg_o0 = dg_b - dg_c;
}
extern "C" void dg_sub2(void)
{
    dg_o0 = dg_c - dg_b;
}

// CHAIN: three operands, left-associated. The within-statement height axis:
// dg_d feeds only the SECOND add, so its address formation and load sit lower
// in the dependence tree than dg_b/dg_c's.
extern "C" void dg_chain(void)
{
    dg_o0 = dg_b + dg_c + dg_d;
}

// LIT: literal producers to globals; dg_o0 and dg_o2 share the value 1.
extern "C" void dg_lit(void)
{
    dg_o0 = 1;
    dg_o1 = 2;
    dg_o2 = 1;
}

// IF: a guarded store with one independent statement available to hoist.
// The compare/branch separation cell (B-RULE-2's mechanism).
extern "C" void dg_if(void)
{
    dg_o0 = dg_d + 1;
    if (dg_a) {
        dg_o1 = 1;
    }
}

// CAP: twenty independent statements — past the scheduler's 0x50-tuple region
// cap, in ONE basic block with no call. If regions cap at 80 tuples, the
// hoisted address-formation cluster must BREAK somewhere before the 20th
// statement; the naive "everything hoists to block top" model predicts one
// unbroken cluster.
extern "C" void dg_cap(void)
{
    dg_p0  = dg_i0  + 1;
    dg_p1  = dg_i1  + 2;
    dg_p2  = dg_i2  + 3;
    dg_p3  = dg_i3  + 4;
    dg_p4  = dg_i4  + 5;
    dg_p5  = dg_i5  + 6;
    dg_p6  = dg_i6  + 7;
    dg_p7  = dg_i7  + 8;
    dg_p8  = dg_i8  + 9;
    dg_p9  = dg_i9  + 10;
    dg_p10 = dg_i10 + 11;
    dg_p11 = dg_i11 + 12;
    dg_p12 = dg_i12 + 13;
    dg_p13 = dg_i13 + 14;
    dg_p14 = dg_i14 + 15;
    dg_p15 = dg_i15 + 16;
    dg_p16 = dg_i16 + 17;
    dg_p17 = dg_i17 + 18;
    dg_p18 = dg_i18 + 19;
    dg_p19 = dg_i19 + 20;
}

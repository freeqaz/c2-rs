// **W11 — guarded EARLY RETURNS in a framed call sequence.** The port's first
// intra-section `b` and its first real label→offset map.
//
// `work/w-conv/PREREG.md` §1 prices all 17 FRONTIER TUs off their own
// disassembly at the workload's flags: the minimum is **6** independent
// refusals and the cheapest framed-and-branching one is **9**, so no TU is a
// target and this rung is picked by *construct*. Ranked that way over the same
// 17 objs, the top two missing mechanisms are:
//
//     a real label->offset map (>=2 transfers, >=1 shared target)   14 TUs
//     the intra-section unconditional `b` (board #191)              10 TUs
//
// and the port had emitted neither. W10 closed the only route to #191 that had
// been tried — the `else` arm — because its block layout is mode-dependent on a
// threshold that is a c2 cost model. **This is a second route and it is not that
// shape**: the `b` targets the epilogue, not a join block.
//
//     /O1  (what the dc3 workload compiles)     /Ox and /O2
//       mflr/stw/stwu                             mflr/stw/stwu
//       cmpwi cr6,r3,0                            cmpwi cr6,r3,0
//       bt    26,+12                              bt    26,+24
//       li    r3,5                                li    r3,5
//       b     +12      <- 48000...                addi/lwz/mtlr/blr  <- COPIED
//       bl    ?v0                                 bl    ?v0
//       li    r3,0                                li    r3,0
//       addi/lwz/mtlr/blr                         addi/lwz/mtlr/blr
//
// That mode split is board row **X-b** and it refutes `docs/OPT_MODE.md`. It is
// **not** W10's declined cost model: the block `/Ox` duplicates here is the
// epilogue, whose length is a constant of the frame class, so there is no
// threshold to fit — only two measured layouts.
//
// **The STRUCTURAL axes are crossed and the value axis is not.** Board #198 and
// w-frame §4.5.3 record the same defect twice — a family exhaustive on the axis
// it varies and blind on the one it holds fixed reads as complete. W9 already
// graded all six relations x both signednesses on the compare/branch encoder
// this file shares, so the relation is sampled here rather than swept, and the
// axes crossed instead are:
//
//     guard COUNT ................. 1, 2, 3        g1  g2  g3
//     result KIND ................. int, void      g*  w1  w2
//     trailing-call COUNT ......... 1, 2, 4        g1  t2  t4
//     scrutinee POSITION .......... r3, r5, r6     g1  p3  x4
//     exit-literal MAGNITUDE ...... 0, 5, 11, 22, -1, 4660, 32767
//     operand SIGNEDNESS .......... int, unsigned, pointer
//
// The **void** rows are not a spelling variant of the int ones: an empty arm is
// a different *branch sense*. c2 deletes the block and points the branch
// straight at the epilogue with the relation itself where the value form emits
// its negation — `w1` is `bf 26` where `g1` is `bt 26`. That is
// `work/w-cross/PREREG.md` §1's empty-arm inversion in the smallest body that
// has it, and it composes (`w2`). The void rows are also **byte-identical at
// `/O1` and `/Ox`**, which makes them the control on the mode split above:
// with no arm there is nothing to duplicate.
//
// Every exit value in a body is DISTINCT, and that is load-bearing rather than
// stylistic — see `w11_early_return_neg.cpp`.

void v0();
void v1();
void v2();
void v3();

// ---- guard COUNT: 1, 2, 3 -------------------------------------------------
// Three `b`s, all naming ONE target. The displacement of each is a function of
// everything after it, which is why this needs a label map and W8/W10 did not.
int g1(int a) { if (a != 0) return 5; v0(); return 0; }
int g2(int a, int b) { if (a != 0) return 5; if (b != 0) return 11; v0(); return 0; }
int g3(int a, int b, int c) {
    if (a != 0) return 5;
    if (b != 0) return 11;
    if (c != 0) return 22;
    v0();
    return 0;
}

// ---- result KIND: void, and its empty-arm inversion ------------------------
void w1(int a) { if (a != 0) return; v0(); v1(); }
void w2(int a, int b) { if (a != 0) return; if (b != 0) return; v0(); v1(); }

// ---- trailing-call COUNT ---------------------------------------------------
int t2(int a) { if (a != 0) return 5; v0(); v1(); return 0; }
int t4(int a) { if (a != 0) return 5; v0(); v1(); v2(); v3(); return 0; }

// ---- scrutinee POSITION: the third and fourth formals ----------------------
int p3(int a, int b, int c) { if (c != 0) return 5; v0(); return 0; }
int x4(int a, int b, int c, unsigned d) { if (d >= 7u) return 5; v0(); return 0; }

// ---- exit-literal MAGNITUDE, and a signed compare against a NEGATIVE one ---
// `-1` and `4660` exercise `li`'s full signed immediate; `-3` and `100` are the
// first non-zero guard literals in this class, one of each sign.
int x5(int a, int b) {
    if (a < -3) return -1;
    if (b > 100) return 4660;
    v0();
    return 0;
}
int x7(int a) { if (a != 0) return 32767; v0(); return 0; }

// ---- operand SIGNEDNESS: a pointer null check is an UNSIGNED compare -------
int r_eq(void *p) { if (p == 0) return 5; v0(); return 0; }

// **w-biquad — the NEGATIVE cells of the float-store diamond and its
// forwarding constructor.**
//
// Every function here is one step outside `wbiquad_fp_store_diamond.cpp` along
// exactly one axis. `NotImplemented` is the only correct verdict for this whole
// file, at `/O1` and at `/Ox` alike.
//
// **The clauses are distinct, and that was checked rather than asserted.** A
// `_neg` file whose cells all trip the same clause tests one thing eleven times
// — the failure `w-blockir` (#2305), `w-json` and `w-park` each paid for in a
// different way. The per-cell key is in `work/w-biquad/NEG_CLAUSES.md`, read
// with a scratch probe patch applied to the recognizers' own decline paths and
// then reverted (`work/w-biquad/decline_probe.md`); board **#1704**'s defect
// again, and the ninth time it has cost a lane a patch.
//
// **The discipline this file exists for**: an arm no cell grades is an arm that
// will be wrong when something finally reaches it (board #1148). Ten of the
// eleven cells below are shapes c2 lowers perfectly well, several of them one
// constant away from the accepted form, and the classes refuse all of them
// because there is no *graded* emitter arm on the other side.
//
// The eleventh, `n_ctor_extern`, is the one whose refusal is not in the reader
// at all: its SHAPE is accepted and `c2_core::comdat` declines it, because
// M-RULE's park register is a fact about the CALLEE and this callee is an
// undefined external. c2 emits 48 bytes for it — `mr r31,r3`, a `std`/`ld` pair
// and a `mr r3,r31` restore — against the accepted class's 36
// (`work/w-biquad/probe/park_extern.cpp`, compiled). That is a REALIZED wrong
// emit and not a hypothetical one: a port that took the shape without the
// callee check would emit the nine-word body here.

namespace WBQN {

    // ---- the POOL axis -------------------------------------------------------

    // ONE constant, not two. B-RULE's two-block `lis` placement is a statement
    // about two pools in two blocks; with one pool there is one `lis` and the
    // whole placement question is a different one. c2 emits it fine — one `lis`
    // in the entry block and six `stfs` out of `f0`.
    struct A { float c[7]; void s(float *f); };
    void A::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=0.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // THREE constants in the then-arm. `WB_CHOOSER_FINDINGS`' cell B5 measures
    // two pools sharing one block taking the scratch **r11 then r10**; a third
    // has no witness at all, and the emitter has no third register to hand out.
    struct B { float c[7]; void s(float *f); };
    void B::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=2.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // The JOIN is not ONE constant. Its first store fixes the entry-hoisted pool
    // `A` — the value `f0` carries across the whole diamond — and a second,
    // different constant in the same block needs a THIRD pool, a second
    // addressing register and a `lis` this class has no block plan for.
    struct C { float c[7]; void s(float *f); };
    void C::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0f; c[5]=3.0f;
    }

    // ---- the DIVISION-RUN axis ----------------------------------------------

    // ONE division. B′-RULE says the reload's operands go in source order in the
    // LAST statement of the run and reversed in every earlier one; with a run of
    // one, "the last" and "the only" coincide and the cell would be admitted on
    // an ambiguity rather than on the rule. `WB_CHOOSER_FINDINGS` §4.1's shortest
    // graded run is two.
    struct D { float c[7]; void s(float *f); };
    void D::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // The divisors DIFFER, so there is no common subexpression, no reload, and
    // nothing for B′-RULE to order. c2 emits ten independent loads.
    struct E { float c[7]; void s(float *f); };
    void E::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[4]; c[2]=f[2]/f[5]; c[3]=f[4]/f[6]; c[4]=f[5]/f[2]; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // ---- the ARM-SIZE axis ---------------------------------------------------

    // ONE then-store, so the block-local constant is also the arm's only
    // constant and the entry-hoisted one is used nowhere in the then-block. The
    // two-pool shape this class transcribes needs both.
    struct F { float c[7]; void s(float *f); };
    void F::s(float *f) {
        if (f == 0) { c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // NO join stores. Then no constant's uses straddle the branch, the
    // entry-block hoist has nothing to dominate, and B-RULE puts BOTH `lis`
    // inside their own arms — a different block plan, not a shorter one.
    struct G { float c[7]; void s(float *f); };
    void G::s(float *f) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
    }

    // ---- the GUARD axis ------------------------------------------------------

    // `!=` instead of `==`. The IL's `38` is branch-on-FALSE, so an inverted
    // relation names the OTHER successor: the arms swap and the emitted `bc`
    // carries the opposite sense. One byte in the IL, two blocks in the obj.
    struct H { float c[7]; void s(float *f); };
    void H::s(float *f) {
        if (f != 0) { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        else { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        c[6]=0.0f; c[5]=0.0f;
    }

    // ---- the WIDTH axis ------------------------------------------------------

    // `double` members. Every store is `stfd`, every load `lfd`, the divide is
    // `fdiv` and not `fdivs`, and each pool is an 8-byte `.rdata` with
    // characteristics `0x40401040` instead of `0x40301040`. Four instruction
    // selections and a section header, none of them graded.
    struct I { double c[7]; void s(double *f); };
    void I::s(double *f) {
        if (f == 0) { c[4]=0.0; c[3]=0.0; c[2]=0.0; c[1]=0.0; c[0]=1.0; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0; c[5]=0.0;
    }

    // ---- the FORMALS axis ----------------------------------------------------

    // A THIRD formal. `this` is r3 and the guarded pointer r4; a second explicit
    // formal is r5 and changes nothing about the emitted words — which is
    // exactly why it must refuse rather than be waved through, because the class
    // has no cell that says so.
    struct J { float c[7]; void s(float *f, int k); };
    void J::s(float *f, int k) {
        if (f == 0) { c[4]=0.0f; c[3]=0.0f; c[2]=0.0f; c[1]=0.0f; c[0]=1.0f; }
        else { c[0]=f[0]/f[3]; c[1]=f[1]/f[3]; c[2]=f[2]/f[3]; c[3]=f[4]/f[3]; c[4]=f[5]/f[3]; }
        c[6]=0.0f; c[5]=(float)k;
    }

    // ---- the CALLEE axis, and the one cell whose refusal is not in the reader -

    // A forwarding constructor whose callee is an UNDEFINED EXTERNAL. The SHAPE
    // is the accepted one; what differs is a fact about the callee, and it
    // changes four of the nine words plus the frame:
    //
    //   accepted (same-TU callee)  mflr · stw · stwu · mr r10,r3 · bl
    //                              · addi · lwz · mtlr · blr            36 B
    //   this cell (external)       mflr · stw · STD R31 · stwu · MR R31,R3 · bl
    //                              · MR R3,R31 · addi · lwz · mtlr · LD R31
    //                              · blr                                48 B
    //
    // Compiled, both sides: `work/w-biquad/probe/park_{local,extern}.cpp`.
    struct K { float c[7]; K(float *f); void s(float *f); };
    K::K(float *f) { s(f); }
}

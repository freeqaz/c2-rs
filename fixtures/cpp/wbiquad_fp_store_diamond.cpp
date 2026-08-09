// **w-biquad — THE NULL-GUARDED FLOAT-STORE DIAMOND and its forwarding
// constructor**, and the first obj this port emits that carries a `.rdata`
// constant pool under `/Gy`.
//
// Both functions below are `src/system/synth_xbox/Biquad.cpp` **verbatim**, with
// the two header includes dropped and the class declaration inlined. That TU is
// the whole positive population of both classes on the 878-TU workload — 2
// functions, 176 `.text` bytes — and it converted at this lane, `match` 20 → 21.
// This file compiles to the same two bodies word for word (140 B / 36 B, checked
// against `work/w-biquad/real.obj` instruction by instruction), which is what
// licenses it as a fixture.
//
// # THE ORDER OF THE TWO FUNCTIONS IS THE WHOLE FIXTURE
//
// `w-blockir` board **#2305**: *"a wrong charge on the LAST function in a TU
// moves nothing after it"*, and that lane's `_neg` cell could not fail until it
// was reordered. The diamond is a **leaf** and has no labels of its own; the
// constructor is **framed** and has a `$M`/`$M`/`$T` triple. So the diamond's
// label surcharge — `+2` per newly pooled FP constant, `docs/LABEL_COUNTER.md`
// §1.1's fourth row — is observable **only** through the constructor's triple,
// and only because the constructor comes second. Reversed, this file would grade
// `match` with the surcharge deleted.
//
// That is not hypothetical here: the surcharge was **missing from
// `plan_labels`** when this lane started, and it had been missing harmlessly
// forever, because every pool-bearing obj the port had ever emitted
// (`w13b_fconst.cpp`, `w13b_fdedup.cpp`, `w13b_fpool.cpp`) is leaves alone,
// where the counter is dead. `Biquad.cpp` is the first TU with both, and the
// constructor's triple came out `$M2570`/`$M2571`/`$T2572` against c2's
// `$M2574`/`$M2575`/`$T2576` — exactly four low, which is `2 + 2`.
//
// # What c2 emits, and the four things a lowering gets wrong
//
//     ?SetCoefficients@Biquad@DSP@@QAAXPAM@Z            140 B, 8 relocations
//       lis    r11,__real@00000000    <- A's `lis`, in the ENTRY block …
//       cmplwi cr6,r4,0
//       lfs    f0,0(r11)              <- … and its `lfs` there too
//       bne    cr6,+0x24
//       lis    r11,__real@3f800000    <- B's `lis`, FIRST WORD OF THE ARM
//       stfs   f0,16(r3) · 12(r3) · 8(r3) · 4(r3)
//       lfs    f13,0(r11)             <- B's `lfs`, AT THE USE, five words down
//       stfs   f13,0(r3)
//       b      +0x54
//       lfs f12,12(r4) · lfs f13,0(r4)  · fdivs · stfs   ┐ divisor FIRST …
//       …                                               │ × 4
//       lfs f13,20(r4) · lfs f12,12(r4) · fdivs · stfs   ┘ … and SWAPPED last
//       stfs   f0,24(r3) · stfs f0,20(r3)                  the join, still in f0
//       blr
//
//     ??0Biquad@DSP@@QAA@PAM@Z                          36 B, 1 relocation
//       mflr/stw/stwu -96 · mr r10,r3 · bl ?SetCoefficients · addi/lwz/mtlr/blr
//
// 1. **B-RULE** (`WB_CHOOSER_FINDINGS` §3.3): one `lis` per pool per function,
//    at the top of the earliest block that DOMINATES every use. Both readings —
//    "dominator" and "the function's first word" — agree on word 0 here and
//    disagree on word 4.
// 2. **The `lfs` is at the use.** B's two halves are five words apart, which is
//    why `FpConstRef::lo_off` is a field and not `hi_off + 4`.
// 3. **B′-RULE** (§4.1): the CSE'd divisor loads first in every statement of the
//    run EXCEPT the last. Two words, at the end of one arm.
// 4. **The constructor's park is `mr r10,r3` with NO restore**, and both halves
//    are statements about the CALLEE. `work/w-biquad/probe/park_extern.cpp` is
//    the same constructor over an undefined external and c2 emits 48 bytes with
//    `mr r31,r3`, a `std`/`ld` pair and `mr r3,r31`.
//
// # `/O1` ONLY, and the gate is in the PARSER
//
// Board **#1638**. `differential()` drives the default `/Ox` profile, where both
// classes refuse and the whole file is `NotImplemented`; the `match` is graded by
// `scripts/mode_lane.sh /O1`. A refusal becoming a wrong emit is strictly worse
// than a gap (board #232), which is why the `/Ox` arm asserts the refusal rather
// than being left ungraded.

namespace DSP {
    class Biquad {
    public:
        Biquad(float *);
        void SetCoefficients(float *);
        float coefs[7];
    };

    void Biquad::SetCoefficients(float *flts) {
        if (flts == 0) {
            coefs[4] = 0.0f;
            coefs[3] = 0.0f;
            coefs[2] = 0.0f;
            coefs[1] = 0.0f;
            coefs[0] = 1.0f;
        } else {
            coefs[0] = flts[0] / flts[3];
            coefs[1] = flts[1] / flts[3];
            coefs[2] = flts[2] / flts[3];
            coefs[3] = flts[4] / flts[3];
            coefs[4] = flts[5] / flts[3];
        }
        coefs[6] = 0.0f;
        coefs[5] = 0.0f;
    }

    Biquad::Biquad(float *flts) { SetCoefficients(flts); }
}

// **Negative** — board **#844**'s composition seam, its five boundaries. The
// reference must compile and the port must honestly return `NotImplemented`:
// never a framed obj where c2 emits a branch, and never a run whose two right
// words are in the wrong order.
//
// Three of these five were found by this file's POSITIVE half grading
// `Port=Mismatch` on its first two runs. They are here rather than narrowed away
// silently, so the boundary cannot be re-crossed without meeting the body that
// crosses it.
//
// `T::tvoid` / `T::tret` / `T::tdiscard` — the `void`, `return <call>` and
//   discarded-`int` forms of the *identical* run and the *identical* call.
//   Measured (`w-heap` §3.3, boards #869/#1131): all three are frame words **0**
//   and c2 **tail-calls behind the run** — `li r11,0 ; stw r5,4(r3) ; b ?Alloc`.
//   Only the constructor's implicit `return this` keeps a value live across the
//   call, and without it there is nothing to preserve. Three of the four cells
//   that look like the composition are a different shape, which is why the
//   constructor tail is required POSITIVELY by the reader.
//
// `AR3` — a **free** callee, so the argument setup is `mr r3,r4` and it destroys
//   `this`. Measured (board #870, `work/w-seam2/grid/sr_c1r3`):
//   `mr 31,3 ; stw 5,16(3) ; li 11,0 ; stw 3,0(3) ; stw 31,4(31) ; mr 3,4 ;
//   stw 11,20(31) ; bl ?g1` — the store base **switches r3 -> r31 mid-run** and
//   the setup **interleaves into it**. Board #1129 refines #870 to "the setup
//   writes r3", with `w-heap`'s `c1b` as the separating cell, and the reader's
//   own gate is stricter still: every slot must already hold `params[i]`.
//
// `LV1` / `LV2` — **THE REFUTATION OF BOARD #866 IN ITS GENERAL FORM**, and the
//   reason the positive file's first version graded `Port=Mismatch`:
//
//     void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }        LEAF
//         li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
//     P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }    FRAMED
//         li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
//
//   **The two unproduced stores swap.** `a` is the call's argument and is live
//   until the `bl`; `b` dies at its own store. The *same* body with a nullary
//   callee does not swap — that is `NL` in the positive file. So "the leaf
//   schedule transfers unchanged into a framed body" (96 cells in `w-seam`, 34
//   more in `w-seam2`'s GRID S) holds only while **no store reads a value the
//   call keeps alive**.
//
//   `LV2` fixes which reading of the rule is right: its callee takes **two**
//   arguments and the run stores the **second** one, so a gate keyed on
//   "argument slot 1" would emit it while a gate keyed on "every slot >= 1"
//   refuses it. Measured (`work/w-seam2/grid4/a2_break2`) it swaps, so the wider
//   gate is the correct one and this is its witness.
//
// `WD` — a **multi-word literal** (`lis`+`ori`), alone in its run. As a leaf that
//   is in class: one live range with nothing to interleave with. In a framed body
//   the `mr r31,r3` lands **between the two halves**:
//
//     WD::WD(unsigned a, unsigned b) { m1=70000; m2=b; p0=this; Alloc(a); }
//         lis 11,1 ; stw 5,8(3) ; stw 3,0(3) ; mr 31,3 ; ori 11,11,4464 ; stw 11,4(3)
//
//   A producer is one slot to the splice, so the splice can never place the copy
//   inside one. The leaf's own "wide literal beside another producer" rule does
//   not predict this, because a leaf has nothing to land there.
//
// The positive half is `w844_store_run_call.cpp`.

struct BE { BE* mNext; BE* mPrev; };
extern BE* gfree(unsigned int);

// ---- the three forms that TAIL-CALL behind the run ------------------------
struct T {
    void tvoid(unsigned int a, unsigned int b);
    BE* tret(unsigned int a, unsigned int b);
    void tdiscard(unsigned int a, unsigned int b);
    BE* Alloc(unsigned int);
    T* p0; unsigned int m1, m2;
};
void T::tvoid(unsigned int a, unsigned int b)    { m1 = 0; m2 = b; Alloc(a); }
BE*  T::tret(unsigned int a, unsigned int b)     { m1 = 0; m2 = b; return Alloc(a); }
void T::tdiscard(unsigned int a, unsigned int b) { m1 = 0; m2 = b; BE* r = Alloc(a); (void)r; }

// ---- a FREE callee: the setup writes r3 and destroys `this` ----------------
struct AR3 { AR3(unsigned int a, unsigned int b); AR3* p0; unsigned int m1, m2; };
AR3::AR3(unsigned int a, unsigned int b) { m1 = 0; m2 = b; gfree(a); }

// ---- the run stores a value the CALL keeps alive: #866 refuted -------------
struct LV1 {
    LV1(unsigned int a, unsigned int b);
    BE* A1(unsigned int);
    unsigned int m0, m1, m2;
};
LV1::LV1(unsigned int a, unsigned int b) { m0 = 0; m1 = b; m2 = a; A1(a); }

struct LV2 {
    LV2(unsigned int a, unsigned int b, unsigned int c);
    BE* A2(unsigned int, unsigned int);
    unsigned int m0, m1, m2;
};
LV2::LV2(unsigned int a, unsigned int b, unsigned int c) { m0 = 0; m1 = c; m2 = b; A2(a, b); }

// ---- a multi-word literal: the `mr` lands between `lis` and `ori` ----------
struct WD { WD(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            WD* p0; unsigned int m1, m2, m3; };
WD::WD(unsigned int a, unsigned int b) { m1 = 70000; m2 = b; p0 = this; Alloc(a); }

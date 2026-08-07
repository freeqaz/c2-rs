// **Positive** — board **#844**, the COMPOSITION SEAM: a scheduled store run as
// the MIDDLE of a framed body. Every function here must emit, and the whole obj
// must be byte-exact.
//
//   H::H(unsigned initSize, unsigned size) { mSize = size; mCount = 0; Alloc(initSize); }
//
//     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
//     li r11,0 ; stw r5,16(r3) ; mr r31,r3 ; stw r11,20(r3)
//     bl ?Alloc@H@@QAAPAUBE@@I@Z
//     mr r3,r31
//     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
//
// ## Why this file is a cross product and not a list
//
// The composition is three shipped models plus one new fact, and the new fact —
// **where `mr r31,r3` sits inside the run** — is a function of *two* counts that
// a list of examples would move together. `w-heap` §5 records exactly that
// mistake happening (#1099's evidence cell had **one** producer where its target
// has two, because it matched the store *count*), so every function below crosses
// the **producer count** against the **run width**, and the width is padded with
// stores that materialise nothing (`m2 = b` is a formal already in a register;
// `p0 = this` is r3). "Six stores" and "two producers" are never the same number
// anywhere in this file.
//
// ## What each function discriminates
//
// `C0` / `C1` / `C2` / `C3` — the producer count 0, 1, 2, 3 at a fixed width.
//   This is the axis board #867's `stores_before_mr = nprod - 1 + min(u, 2)` is
//   *linear* in, so a fixture set at one producer count would pass with the term
//   dropped entirely.
//
// `W1` / `W3` / `W5` — the run width at a fixed producer count, which is the
//   `min(u, 2)` term. `W1` and `W3` straddle the cap and `W5` is past it, so a
//   lowering that used `u` uncapped puts the copy three words late on `W5` and is
//   byte-exact on `W1`.
//
// `U2` — **one** literal stored **twice**. Equal constants CSE to a single `li`,
//   so this is one producer and two stores: it separates "count the producers"
//   from "count the produced stores", which agree in every other cell here. A
//   lowering that counted stores puts the copy one word late.
//
// `NL` — the call takes **no** argument at all (`Reset()` against
//   `Alloc(initSize)`). Both have an EMPTY argument setup, which is the regime
//   the composition is admitted on (board #1129), and this pins that "empty
//   setup" is not read off the argument *count*. It is also the one form in which
//   a run may store **every** formal, because a nullary call keeps nothing alive.
//
// `BW` — `stb` / `sth` / `stw` / `std` widths inside one run, so the composition
//   cannot be passing by emitting `stw` for everything.
//
// `H` — `xboxheap.cpp`'s own shape minus the two sub-object addresses its reader
//   still refuses (#836/#868): a literal, a formal and two `this` stores before a
//   member call on `this`.
//
// ## THE AXIS THAT REFUTED THIS FILE'S FIRST VERSION
//
// Every unproduced store here stores either `this` or the **last** formal, and
// never the formal the call passes. That is not tidiness, it is the class
// boundary:
//
//   void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }        LEAF
//       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
//   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }    FRAMED
//       li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
//
// **The two unproduced stores swap**, so board #866's *"the leaf schedule
// transfers unchanged into a framed body"* is false in general — it holds only
// while no store reads a value the call keeps alive. This file's first version
// stored the call's own argument in six of its bodies and graded
// `Port=Mismatch`; those cells now live in `w844_store_run_call_neg.cpp`, where
// the port must refuse them.
//
// ## The negatives are `w844_store_run_call_neg.cpp`
//
// A fixture obj is byte-compared whole, so one refused body would refuse the TU
// and this file would grade nothing. The five boundaries live there.

struct BE { BE* mNext; BE* mPrev; };

struct H {
    H(unsigned int initSize, unsigned int size);
    BE* Alloc(unsigned int);

    H* mFreeHead;          // 0
    H* mUsedHead;          // 4
    BE mListHead;          // 8  (mNext 8, mPrev 12)
    unsigned int mSize;    // 16
    unsigned int mCount;   // 20
    BE mSecond;            // 24 (mNext 24, mPrev 28)
    unsigned int mFlags;   // 32
    unsigned int mPeak;    // 36
};

// ---- the producer-count axis, at a fixed width -----------------------------
struct C0 { C0(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            C0* p0; unsigned int m1, m2, m3, m4, m5; };
C0::C0(unsigned int a, unsigned int b) { p0 = this; m1 = b; m2 = b; Alloc(a); }

struct C1 { C1(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            C1* p0; unsigned int m1, m2, m3, m4, m5; };
C1::C1(unsigned int a, unsigned int b) { m1 = 0; p0 = this; m2 = b; Alloc(a); }

struct C2 { C2(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            C2* p0; unsigned int m1, m2, m3, m4, m5; };
C2::C2(unsigned int a, unsigned int b) { m1 = 0; m2 = 7; p0 = this; m3 = b; Alloc(a); }

struct C3 { C3(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            C3* p0; unsigned int m1, m2, m3, m4, m5; };
C3::C3(unsigned int a, unsigned int b)
{ m1 = 0; m2 = 7; m3 = 13; p0 = this; m4 = b; Alloc(a); }

// ---- the WIDTH axis, at a fixed producer count -----------------------------
struct W1 { W1(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            W1* p0; unsigned int m1, m2, m3, m4, m5, m6; };
W1::W1(unsigned int a, unsigned int b) { m1 = 0; m2 = b; Alloc(a); }

struct W3 { W3(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            W3* p0; unsigned int m1, m2, m3, m4, m5, m6; };
W3::W3(unsigned int a, unsigned int b) { m1 = 0; m2 = b; p0 = this; m3 = b; Alloc(a); }

struct W5 { W5(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            W5* p0; W5* p1; unsigned int m1, m2, m3, m4, m5, m6; };
W5::W5(unsigned int a, unsigned int b)
{ m1 = 0; m2 = b; p0 = this; m3 = b; p1 = this; m4 = b; Alloc(a); }

// ---- one literal, TWO uses: a producer count that is not a store count -----
struct U2 { U2(unsigned int a, unsigned int b); BE* Alloc(unsigned int);
            U2* p0; unsigned int m1, m2, m3; };
U2::U2(unsigned int a, unsigned int b) { m1 = 0; m2 = 0; m3 = b; Alloc(a); }

// ---- xboxheap's own shape, minus the sub-object addresses ------------------
H::H(unsigned int initSize, unsigned int size)
{
    mSize = size;
    mCount = 0;
    mFreeHead = this;
    mUsedHead = this;
    Alloc(initSize);
}

// ---- the NULLARY call: nothing is live, so every formal is storable --------
struct NL { NL(unsigned int a, unsigned int b); BE* Reset();
            NL* p0; unsigned int m1, m2, m3; };
NL::NL(unsigned int a, unsigned int b) { m1 = 0; m2 = b; m3 = a; p0 = this; Reset(); }

// ---- mixed store widths inside the run ------------------------------------
struct BW {
    BW(unsigned int a, unsigned int b);
    BE* Alloc(unsigned int);
    unsigned char mb;   // 0
    unsigned short mh;  // 2
    unsigned int mw;    // 4
    long long mq;       // 8
    BW* pp;             // 16
};
BW::BW(unsigned int a, unsigned int b)
{
    mb = 1;
    mh = 2;
    mw = b;
    mq = 3;
    pp = this;
    Alloc(a);
}

// ---- the LEAF control: the identical run with NO call ----------------------
// The shipped store-run emitter already handles this, byte-exactly. It is here
// so this file fails if the seam ever drifts the leaf: the framed run minus its
// `mr r31,r3` must be the leaf's run, word for word — board #866 *inside the
// class this file stays in*, re-measured over 34 cells of `work/w-seam2/grid/`
// and again on GRID S4's five leaf controls.
//
// The wide-literal cell that used to sit here is in the negative file: the
// `mr r31,r3` lands BETWEEN the `lis` and the `ori`, which the leaf's own
// rule does not predict because the leaf has nothing to land there.
struct LF { void set(unsigned int a, unsigned int b); LF* p0; unsigned int m1, m2; };
void LF::set(unsigned int a, unsigned int b) { m1 = 0; m2 = b; p0 = this; }

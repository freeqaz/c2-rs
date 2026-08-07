// **Board #1212 — `mr r31,r3` IS PLACED BY #584's LEADING RUN, NOT BY THE
// COUNT**, positive fixture.
//
// A store run whose base is a C++ reference bind, followed by board #1129's
// constructor call tail. `w-carrier` shipped this family as a REFUSAL
// (`store-run-bind-call-tail-mr-slot`) after four `88-store-run-call` sweep
// cases and 56 cross cells graded `Port=Mismatch` on its first emitter — and
// named the correction rather than taking it, because it changes a rule that
// governs every #844 body and would have rested on those same four cells.
//
// The mechanism, from `w-carrier`'s own bisect (`work/w-carrier/bisect/s1427.cpp`):
//
//     H::H(unsigned a, unsigned b) { BE& lh = mListHead; mCount = 0;
//                                    lh.mNext = (BE*)this; Reset(); }
//     real c2:  li 11,0 ; mr 31,3 ; stw 11,20(3) ; stw 3,8(3) ; bl
//     the port: li 11,0 ; stw 11,20(3) ; mr 31,3 ; stw 3,8(3) ; bl   WRONG
//
// `codegen::store_run_call::save_slot` was fed the COUNT of stores that
// materialise nothing. Its own module doc argued that equals board #584's `u`,
// the LEADING RUN of such stores in the FINAL order — *"they cannot be
// [separated]"* — and that argument is `store_order`'s floor, true on a
// SINGLE-symbol run and false once the run has two base symbols. A reference
// bind is a second base symbol (board **#1128**), so the cross-symbol pin can
// strand an unproduced store behind a produced one; the leading run stops there
// and the count keeps counting.
//
// `w-mrslot` graded the swap on GRID R — 145 cells sha256'd before the first
// `cl.exe`, 93 with an observed `mr r31,r3`, 30 of them separating the two
// readings, every quantity read out of real `c2.dll`'s own emitted words:
// **the leading run is 93 HIT / 0 MISS and the count is 63 / 30.**
//
// Every function below is one point of that grid, chosen so the fixture alone
// separates the rivals. `u` is the leading run; `nprod` the distinct producers;
// COUNT is what the old rule would have emitted.
//
//   fn_lead0   nprod 1, u 0, count 1   COUNT says 1, c2 says 0   SEPARATES
//   fn_lead1   nprod 1, u 1, count 1   both say 1               control
//   fn_lead2   nprod 1, u 2, count 2   both say 2               control
//   fn_lead0b  nprod 1, u 0, count 2   COUNT says 2, c2 says 0   SEPARATES
//   fn_lead1b  nprod 1, u 1, count 2   COUNT says 2, c2 says 1   SEPARATES
//   fn_nopool  nprod 0, u 2            the empty-pool domain, unmoved by the swap
//   fn_arity1  nprod 1, u 0            a ONE-ARGUMENT callee (board #1189 —
//                                      the schedule is not monotone in liveness,
//                                      so arity is varied and not reasoned about)
//   fn_basebind  the bind hangs off a FORMAL the call keeps ALIVE — board #1215
//                deleted a clause for this case as dead, and it is NOT restored:
//                this function is byte-exact, so a clause refusing it would
//                refuse an obj the port gets right
//   fn_s1427   `w-carrier`'s own bisect cell, the body that refuted its emitter
//
// `w1199_bind_run_neg.cpp` carries the four clauses that are still refusals; it
// had five and the fifth was this family.

struct BE { unsigned f0; unsigned f1; unsigned f2; unsigned f3; };

struct H {
    H* mLink;          // 0
    unsigned mA;       // 4
    BE mBlk;           // 8   f0@8 f1@12 f2@16 f3@20
    unsigned mB;       // 24
    unsigned mC;       // 28
    unsigned mD;       // 32
    H(unsigned p, unsigned q);
    H(unsigned p, unsigned q, int);
    H(unsigned p, unsigned q, char);
    H(unsigned p, unsigned q, short);
    H(unsigned p, unsigned q, long);
    H(unsigned p, unsigned q, unsigned char);
    H(H* w, unsigned q);
    BE* Grab(unsigned n);
    BE* Take(H* n);
    BE* Reset();
};

// nprod 1, u 0, count 1 — the produced store leads, so the copy lands after
// ZERO stores where the count reading says one.
H::H(unsigned p, unsigned q) {
    BE& r = mBlk;
    mA = 0u;
    r.f0 = q;
    Reset();
}

// nprod 1, u 1, count 1 — the unproduced store leads and the two readings agree.
H::H(unsigned p, unsigned q, int) {
    BE& r = mBlk;
    mA = q;
    r.f0 = 0u;
    Reset();
}

// nprod 1, u 2, count 2 — two leading unproduced stores, both readings say 2.
H::H(unsigned p, unsigned q, char) {
    BE& r = mBlk;
    r.f0 = q;
    r.f1 = q;
    mA = 0u;
    Reset();
}

// nprod 1, u 0, count 2 — the widest separation on this grid: the count reading
// puts the copy after TWO stores and c2 puts it after none.
H::H(unsigned p, unsigned q, short) {
    BE& r = mBlk;
    mA = 0u;
    r.f0 = q;
    mB = q;
    Reset();
}

// nprod 1, u 1, count 2 — the unproduced store leads but the second one is
// stranded behind the produced one, so the readings differ by one.
H::H(unsigned p, unsigned q, long) {
    BE& r = mBlk;
    r.f0 = q;
    mA = 0u;
    mB = q;
    Reset();
}

// nprod 0, u 2 — the run materialises nothing. Every store is unproduced, so no
// produced store can precede one and the leading run IS the capped count: the
// correction provably does not move `REFUSED_EMPTY_POOL`'s boundary, and this
// is the graded witness for the side of it that emits.
H::H(unsigned p, unsigned q, unsigned char) {
    BE& r = mBlk;
    r.f0 = q;
    mA = q;
    Reset();
}

// A ONE-ARGUMENT callee whose actual already occupies its slot, so the call
// emits no move and the run's base register is never written (#1129's `c0`).
BE* Grab2(H* h, unsigned p, unsigned q);
struct J {
    J* mLink;
    unsigned mA;
    BE mBlk;
    unsigned mB;
    J(unsigned p, unsigned q);
    BE* Grab(unsigned n);
};
J::J(unsigned p, unsigned q) {
    BE& r = mBlk;
    mA = 0u;
    r.f0 = q;
    Grab(p);
}

// The bind hangs off a FORMAL the trailing call keeps ALIVE. Board #1215
// deleted a `live-argument-base` clause as dead because the call-tail refusal
// took every body it could have caught; that refusal is lifted, and this
// function is why the clause is still not restored.
H::H(H* w, unsigned q) {
    BE& r = w->mBlk;
    mA = 0u;
    r.f0 = q;
    Take(w);
}

// `work/w-carrier/bisect/s1427.cpp` itself — the body that graded
// `Port=Mismatch` and bought the refusal this fixture retires.
struct K {
    K* mFreeHead;
    K* mUsedHead;
    BE mListHead;
    unsigned mSize;
    unsigned mCount;
    K(unsigned initSize, unsigned size);
    BE* Reset();
};
K::K(unsigned initSize, unsigned size) {
    BE& lh = mListHead;
    mCount = 0;
    lh.f0 = size;
    Reset();
}

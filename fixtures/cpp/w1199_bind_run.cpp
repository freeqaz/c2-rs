// **Board #1199 — THE BIND CARRIER**, positive fixture.
//
// A C++ reference (or pointer) local bound to a formal's interior —
// `auto& listHead = mListHead;` — used in a store run's BASE position. `c1xx`
// spells it as a store of an address into a `.sy` automatic, and the local's
// token then stands where a formal's would.
//
// The carrier is `IlOp::BoundAddr { tok, base, off }`: the token, the formal it
// hangs off, and the offset. The token is the store's base SYMBOL, `base + off`
// is its address, and the two are derivations of ONE value — which is what keeps
// the bind spelling apart from the direct one. Board **#1128** measured that
// `src/xdk/nuispeech/xboxheap.cpp`'s constructor written with and without the
// bind emits DIFFERENT bodies, so a reader that resolved the local to the formal
// would hand the emitter the other body's op stream (board #232's direction).
//
// Every function here is the BIND spelling beside its DIRECT twin, so the pair is
// in one obj: `fn_*` binds, `dr_*` does not. Six of the pairs emit differently in
// real `c2.dll` and every one of the twelve is byte-exact.
//
// The accept class is narrow on purpose and each clause is measured:
//   * the bound name is used ONLY as a base — as a VALUE it is an interior
//     address, a register-derived producer, and beside a literal that is the
//     mixed-kind run `codegen::alloc` refuses (boards #836/#868/#1134);
//   * at most ONE distinct producer — at one producer `order::store_order`
//     provably cannot refuse; at two it can (`work/w-carrier/grid/k_2const`);
//   * at most two base-symbol group crossings — `order::MAX_SYMBOL_CROSSINGS`;
//   * no trailing call — the composition's copy-slot rule is fed the COUNT of
//     unproduced stores where a multi-symbol run needs the leading run, and
//     three graded objs bought that refusal.
// `w1199_bind_run_neg.cpp` is the negative for every one of them.

struct BE { BE* mNext; BE* mPrev; };
struct W { char c0; char c1; short h0; short h1; long long q0; long long q1; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8   (mNext at 8, mPrev at 12)
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    BE mSecond;        // 24  (mNext at 24, mPrev at 28)
    W  mWide;          // 32  (c0 32, c1 33, h0 34, h1 36, q0 40, q1 48)
};

// ONE use, one literal producer. The BIND keeps source order (two symbols pin
// the run); the DIRECT twin moves the produced store out of position 0.
void fn_base1(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    l.mNext = p;
}
void dr_base1(H* h, BE* p) {
    h->mSize = 2;
    h->mListHead.mNext = p;
}

// TWO uses of the bound name.
void fn_base2(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mListHead;
    l.mNext = p;
    l.mPrev = p;
}
void dr_base2(H* h, BE* p) {
    h->mSize = 2;
    h->mListHead.mNext = p;
    h->mListHead.mPrev = p;
}

// NO producer anywhere — the run is all formals and `this`.
void fn_noprod(H* h, BE* p) {
    h->mFreeHead = h;
    BE& l = h->mListHead;
    l.mNext = p;
    l.mPrev = p;
}
void dr_noprod(H* h, BE* p) {
    h->mFreeHead = h;
    h->mListHead.mNext = p;
    h->mListHead.mPrev = p;
}

// THREE stores on the other symbol between the bind and its first use — the axis
// `order::layout_slots` computes its symbol-crossing count over.
void fn_gap3(H* h, BE* p) {
    BE& l = h->mListHead;
    h->mSize = 2;
    h->mFreeHead = h;
    h->mUsedHead = h;
    l.mNext = p;
}
void dr_gap3(H* h, BE* p) {
    h->mSize = 2;
    h->mFreeHead = h;
    h->mUsedHead = h;
    h->mListHead.mNext = p;
}

// A different displacement — the SUM `24 + 0` and `24 + 4` is formed at exactly
// one site in the emitter, never in the reader.
void fn_off24(H* h, BE* p) {
    h->mSize = 2;
    BE& l = h->mSecond;
    l.mNext = p;
    l.mPrev = p;
}
void dr_off24(H* h, BE* p) {
    h->mSize = 2;
    h->mSecond.mNext = p;
    h->mSecond.mPrev = p;
}

// Bound off the SECOND pointer formal, not `this`.
void fn_nonthis(H* a, H* b, BE* q) {
    a->mSize = 2;
    BE& l = b->mListHead;
    l.mNext = q;
}

// TWO binds, off two formals.
void fn_twobinds(H* a, H* b, BE* q) {
    BE& l = a->mListHead;
    BE& m = b->mListHead;
    l.mNext = q;
    m.mNext = q;
}

// A POINTER local rather than a reference — board #1203: `c1xx` spells them as
// the same `.ex` statement, so #839 is not about references at all.
void fn_ptrlocal(H* h, BE* p) {
    h->mSize = 2;
    BE* l = &h->mListHead;
    l->mNext = p;
    l->mPrev = p;
}

// The store WIDTH through a bound base: `stb`, `sth` and DS-form `std`. One
// literal value throughout, so the run keeps its single producer.
void fn_widths(H* h) {
    W& w = h->mWide;
    w.c0 = 0;
    w.h0 = 0;
    w.q0 = 0;
}

// Two symbol-group crossings — one step INSIDE `MAX_SYMBOL_CROSSINGS`, so the
// bound is a boundary and not a blanket.
void fn_cross2(H* h, BE* p) {
    BE& l = h->mListHead;
    h->mSize = 2;
    l.mNext = p;
    l.mPrev = p;
    h->mFreeHead = h;
}

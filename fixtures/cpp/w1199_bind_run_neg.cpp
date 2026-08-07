// **Board #1199 — THE BIND CARRIER**, negative fixture: one function per clause
// of the accept boundary, and every one of them must be **0 of N in class**.
//
// A refusal that refuses nothing is indistinguishable from no refusal at all
// (board **#1175**), so each clause here has a witness the census can count, and
// each carries its own key so its residue stays separately sizeable.
//
// The one that matters most is `MIXED` — it is `src/xdk/nuispeech/xboxheap.cpp`'s
// own shape, the frontier's cheapest TU, and the first time boards **#836/#868**
// are a countable row rather than an argument: over the 878-TU workload
// `store-run-bind-mixed-kind-alloc` is **1**, and it is that function.
//
// **This fixture had FIVE clauses and has FOUR** — board **#1212**, `w-mrslot`.
// The fifth was the trailing call (`store-run-bind-call-tail-mr-slot`), and it
// was a refusal only because `codegen::store_run_call::save_slot` was fed the
// COUNT of unproduced stores where a multi-symbol run needs board #584's LEADING
// RUN. It is fed the leading run now, the clause is gone, and its function moved
// to `w1212_bind_run_call.cpp` where it is graded as a POSITIVE. A retired
// clause leaves this file rather than staying in it with a flipped verdict,
// because the header's claim — *every one of them must be 0 of N in class* — is
// what makes the file readable at a glance.

struct BE { BE* mNext; BE* mPrev; };
struct H {
    H* mFreeHead;      // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
    unsigned mA;       // 24
    unsigned mB;       // 28
    BE mSecond;        // 32
    H(unsigned initSize, unsigned size);
    BE* AllocatePageBlock(unsigned n);
};

// `store-run-bind-mixed-kind-alloc` — the bound name in a store's VALUE
// position, which is an interior address (`addi rD,rBase,off`), beside a
// literal. `codegen::alloc::allocate` refuses a mixed-kind run wholesale: over
// 81 mixed cells clause 1 alone is wrong on 29 and the refusal is wrong on 0
// (#836); the narrow lift is 12 MISS of 36 (#868); and clause 1 is refuted on
// this very mix by `w-heap`'s `j1_lit2` (#1134).
void nf_mixed(H* h) {
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
    h->mCount = 0;
}

// `store-run-bind-address-producer` — the same with no literal, so #836's mix
// does not apply. Refused for a reason of its own: this body's DIRECT twin is
// byte-IDENTICAL and the direct twin is refused (an F2 address-valued group is
// four ops where the emitter models three), and emitting one half of a pair
// whose objs are identical is a divergence with no grid behind it.
void nf_addrprod(H* h) {
    BE& l = h->mListHead;
    l.mNext = &l;
    l.mPrev = &l;
}

// `store-run-bind-multi-producer` — two distinct literals beside a bound base.
// At ONE producer `order::store_order`'s walk provably cannot fail; at two it
// can, and `work/w-carrier/grid/k_2const` is a cell where the model *does*
// answer and this reader declines it anyway, which is the safe direction.
void nf_twoprod(H* h, BE* p) {
    h->mA = 2;
    h->mB = 3;
    BE& l = h->mListHead;
    l.mNext = p;
}

// `store-run-bind-symbol-crossings` — h, l, h, l is THREE base-symbol group
// boundaries, one past `order::MAX_SYMBOL_CROSSINGS`, where `layout_slots` is
// 98.6 % rather than exact and board #621 refused a 99.44 % rival.
void nf_cross3(H* h, BE* p) {
    BE& l = h->mListHead;
    h->mSize = 2;
    l.mNext = p;
    h->mFreeHead = h;
    l.mPrev = p;
}

// The fifth clause was here — `store-run-bind-call-tail-mr-slot`, the trailing
// call. Board **#1212** corrected it rather than kept it, so the function is a
// POSITIVE now and lives in `w1212_bind_run_call.cpp`. The declaration of
// `H::H` above stays so this TU still names the constructor the moved function
// defines; it is deliberately left UNDEFINED here, which is what keeps this
// file's every-function-refuses property true.

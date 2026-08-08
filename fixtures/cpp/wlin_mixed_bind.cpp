// **w-lineage** — the mixed-kind store run `codegen::alloc` SERVES, and it is
// `src/xdk/nuispeech/xboxheap.cpp`'s own shape.
//
// A run mixing a register-derived producer (an interior address) with a
// constant one was refused wholesale for five lanes and twelve allocation keys,
// and it was right to be: every reading of the `d` bonus in
// `cu <= ru + 1 + d` is refuted, GRID L (`work/w-lineage/gridL`, 83 cells frozen
// with their `sha256` before one was compiled) refuted the last five at once,
// and a refusal is wrong on 0 cells while the best rule on record is wrong on 7.
//
// What ships is not a rule. It is a refusal boundary drawn where the disputed
// term is **provably zero**:
//
//     the address's stores go through THE BIND THAT NAMES THE ADDRESS
//       -> the value and store roots are ONE TOKEN, so there is no `d` term;
//       -> and the literal necessarily stores through a DIFFERENT base symbol,
//          where docs/SYMBOL.md's cross-symbol pin forbids reordering, so the
//          store order is forced rather than modelled.
//
// Both functions below are that shape and both are byte-exact against real
// `c2.dll` at `/O1 /Oi /EHsc /GR`. Its separating control is
// `wlin_mixed_bind_neg.cpp`, whose runs differ from these only in **where the
// address's stores are written**, and which must refuse at every lane.
struct BE { BE* mNext; BE* mPrev; };
struct Heap {
    Heap* mFreeHead;
    Heap* mUsedHead;
    BE  mListHead;
    unsigned mSize;
    unsigned mCount;
    Heap(unsigned initSize, unsigned size);
    BE* Allocate(unsigned n);
};

// `xboxheap.cpp`'s ctor in its shipped spelling: the address at 2 uses beside a
// literal at 1, a trailing call, and the bind as the address's own store base.
// `cu = 1 <= ru + 1 = 3`, so the address takes POOL_TOP — `addi r11,r3,8`.
Heap::Heap(unsigned initSize, unsigned size) {
    mSize = size;
    mFreeHead = this;
    mCount = 0;
    mUsedHead = this;
    auto& listHead = mListHead;
    listHead.mNext = &listHead;
    listHead.mPrev = &listHead;
    Allocate(initSize);
}

// The same shape with no call tail and the counts the other side of the
// frontier: the literal at 4 uses against the address at 2, so `cu = 4 > ru + 1`
// and the CONSTANT takes POOL_TOP instead. Two cells of one clause, so the
// fixture cannot pass by getting the frontier's direction wrong.
struct Pair { Pair* n0; Pair* n1; };
struct Box {
    int g0; int g1; int g2; int g3;
    Pair hub;
};
void wlin_far(Box* b) {
    Pair& h = b->hub;
    b->g0 = 3;
    b->g1 = 3;
    b->g2 = 3;
    b->g3 = 3;
    h.n0 = &h;
    h.n1 = &h;
}

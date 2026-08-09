// W-POOL2 — `src/system/utl/Pool.cpp`, the whole translation unit.
//
// Three leaves and 132 bytes of PowerPC: an intrusive free list's constructor,
// its POP and its PUSH. The class declaration is inlined here (the workload TU
// includes `Pool.h`); nothing else differs from the dc3 source, and the
// FUNCTION ORDER is the workload's, because the emitted obj is three `/Gy`
// COMDATs in exactly this order.
//
// `/O1` ONLY. At `/Ox` this is a different obj — the constructor is twenty-ONE
// words with the register plan r9/r10/r8/r7, and `Alloc` stops folding its
// guard to a `bclr` altogether (band 3, two `blr`s, seven words). Both were
// captured on this lane's own `work/w-pool2/ref/PoolOx.obj`. So the `/Ox` lanes
// grade this file as a clean `codegen-gap`, never a `mismatch`, and the mode
// word is asked in the PARSER before any body byte is read (board #1638).
//
// Every constant below is load-bearing and two of them were graded on a probe
// pair rather than assumed:
//
//   * `return nullptr` must be the literal ZERO. `return (void *)1` is
//     **36 bytes** — `bf 26,+12 ; li r3,1 ; blr` — the guard stops folding
//     entirely (`work/w-pool2/probe/p_ret1.obj`).
//   * `count > 1` must be the literal ONE. `count > 0` is **76 bytes** and c2
//     emits a record-form **`divw.`**, folding the comparison back into the
//     division's own opcode and branching on cr0 (`p_gt0.obj`).

class Pool {
public:
    Pool(int, void *, int);
    void *Alloc();
    void Free(void *);

private:
    char *mFree;
};

Pool::Pool(int i1, void *v, int i2) : mFree((char *)v) {
    char *ptr = (char *)v;
    int stride = (i1 + 3) & ~3;
    int count = i2 / stride;
    if (count > 1) {
        int n = count - 1;
        do {
            char *next = ptr + stride;
            *(char **)ptr = next;
            ptr = next;
        } while (--n);
    }
    *(char **)ptr = 0;
}

void *Pool::Alloc() {
    void *ptr = mFree;
    if (!ptr)
        return nullptr;
    mFree = *(char **)ptr;
    return ptr;
}

void Pool::Free(void *v) {
    if (!v) {
        return;
    }
    *(void **)v = mFree;
    mFree = (char *)v;
}

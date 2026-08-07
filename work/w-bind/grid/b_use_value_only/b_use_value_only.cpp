// GRID BIND cell `b_use_value_only`
// AXIS B — the bound name used ONLY as a VALUE, never in a base position.
// This is the half `parse_store_stmt` already refuses through F2 rather
// than through the base gate, so it isolates obligation 1 from obligation
// 2 (w-f23 §5.1).
struct BE { BE* mNext; BE* mPrev; };
struct H {
    BE* mSpare;        // 0
    H* mUsedHead;      // 4
    BE mListHead;      // 8
    unsigned mSize;    // 16
    unsigned mCount;   // 20
};

void fn(H* h) {
    BE& l = h->mListHead;
    h->mSpare = &l;
    h->mSize = 3;
}

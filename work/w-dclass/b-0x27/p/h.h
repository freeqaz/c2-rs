struct E { E *mNext; E *mPrev; };
struct H {
    H *mFreeHead;   // 0x00
    H *mUsedHead;   // 0x04
    E mListHead;    // 0x08
    unsigned mSize; // 0x10
    unsigned mCount;// 0x14
    H(unsigned initSize, unsigned size);
    E *AllocatePageBlock(unsigned);
};

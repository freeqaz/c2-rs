#include "h.h"
H::H(unsigned initSize, unsigned size) {
    mSize = size; mFreeHead = this; mCount = 0; mUsedHead = this;
    mListHead.mNext = &mListHead; mListHead.mPrev = &mListHead;
    AllocatePageBlock(initSize);
}

#include "h.h"
H::H(unsigned initSize, unsigned size) {
    mSize = size; mFreeHead = this; mUsedHead = this;
    E &l = mListHead; l.mNext = &l; l.mPrev = &l;
    AllocatePageBlock(initSize);
}

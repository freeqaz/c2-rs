#include "h.h"
H::H(unsigned initSize, unsigned size) {
    mSize = size; mFreeHead = this; mUsedHead = this;
    mListHead.mNext = &mListHead;
}

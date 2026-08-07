struct BE { BE* mNext; BE* mPrev; };
struct H {
  H(unsigned int initSize, unsigned int size);
  BE* Alloc(unsigned int);
  H* mFreeHead; H* mUsedHead; BE mListHead;
  unsigned int mSize; unsigned int mCount; BE mSecond;
  unsigned int mFlags; unsigned int mPeak;
  unsigned int mA; unsigned int mB; unsigned int mC; unsigned int mD;
};
H::H(unsigned int initSize, unsigned int size) {
  mCount = 0;
  mFlags = 7;
  mFreeHead = this;
  mUsedHead = this;
  mSize = size;
  mA = initSize;
  Alloc(initSize);
}

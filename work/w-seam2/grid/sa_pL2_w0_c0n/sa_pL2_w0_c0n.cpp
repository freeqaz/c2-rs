struct BE { BE* mNext; BE* mPrev; };
extern BE* g1(unsigned int);
struct H {
  H(unsigned int initSize, unsigned int size);
  void mv(unsigned int initSize, unsigned int size);
  BE* mr(unsigned int initSize, unsigned int size);
  void md(unsigned int initSize, unsigned int size);
  BE* Alloc(unsigned int);
  BE* Reset();
  H* mFreeHead; H* mUsedHead; BE mListHead;
  unsigned int mSize; unsigned int mCount; BE mSecond;
  unsigned int mFlags; unsigned int mPeak;
};
H::H(unsigned int initSize, unsigned int size) {
  mCount = 0;
  mFlags = 7;
  Reset();
}

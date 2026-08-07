struct BE { BE* mNext; BE* mPrev; };
extern BE* g1(unsigned int);
extern BE* g2(unsigned int, unsigned int);
extern BE* ga(BE*);
struct H {
  H(unsigned int initSize, unsigned int size);
  H(unsigned int initSize, unsigned int size, H* q);
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
  BE& lh = mListHead;
  mCount = 0;
  lh.mNext = (BE*)this;
  lh.mPrev = (BE*)this;
  Alloc(initSize);
}

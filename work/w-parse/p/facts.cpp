// w-parse: the three parse facts of xboxheap.cpp, re-derived by
// construction against THIS tree.  One self-contained struct per cell --
// deriving them from a common base injects `expr-intrinsic-base-member-addr`
// into every row and hides the fact being measured (measured, first try).
// [0] BASE  formal + this only
struct E0 { E0* mNext; E0* mPrev; };
struct H0 { H0* mFreeHead; H0* mUsedHead; E0 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H0(unsigned, unsigned); };
H0::H0(unsigned a, unsigned b) { mSize = b; mFreeHead = this; mUsedHead = this; }
// [1] F1    literal MIXED into the run
struct E1 { E1* mNext; E1* mPrev; };
struct H1 { H1* mFreeHead; H1* mUsedHead; E1 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H1(unsigned, unsigned); };
H1::H1(unsigned a, unsigned b) { mSize = b; mFreeHead = this; mCount = 0; mUsedHead = this; }
// [2] F1m   the mixture, two stmts
struct E2 { E2* mNext; E2* mPrev; };
struct H2 { H2* mFreeHead; H2* mUsedHead; E2 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H2(unsigned, unsigned); };
H2::H2(unsigned a, unsigned b) { mSize = b; mCount = 0; }
// [3] F1c   CONTROL literal alone
struct E3 { E3* mNext; E3* mPrev; };
struct H3 { H3* mFreeHead; H3* mUsedHead; E3 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H3(unsigned, unsigned); };
H3::H3(unsigned a, unsigned b) { mCount = 0; }
// [4] F2    member ADDRESS as a value
struct E4 { E4* mNext; E4* mPrev; };
struct H4 { H4* mFreeHead; H4* mUsedHead; E4 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H4(unsigned, unsigned); };
H4::H4(unsigned a, unsigned b) { mSize = b; mListHead.mNext = &mListHead; }
// [5] F2r   the same through a bind
struct E5 { E5* mNext; E5* mPrev; };
struct H5 { H5* mFreeHead; H5* mUsedHead; E5 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H5(unsigned, unsigned); };
H5::H5(unsigned a, unsigned b) { mSize = b; E5& l = mListHead; l.mNext = &l; }
// [6] F3    a call AFTER a store run
struct E6 { E6* mNext; E6* mPrev; };
struct H6 { H6* mFreeHead; H6* mUsedHead; E6 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H6(unsigned, unsigned); };
H6::H6(unsigned a, unsigned b) { mSize = b; mFreeHead = this; AllocatePageBlock(a); }
// [7] F3c   CONTROL the call alone
struct E7 { E7* mNext; E7* mPrev; };
struct H7 { H7* mFreeHead; H7* mUsedHead; E7 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H7(unsigned, unsigned); };
H7::H7(unsigned a, unsigned b) { AllocatePageBlock(a); }
// [8] TGT   all three at once
struct E8 { E8* mNext; E8* mPrev; };
struct H8 { H8* mFreeHead; H8* mUsedHead; E8 mListHead; unsigned mSize; unsigned mCount;
  void AllocatePageBlock(unsigned); H8(unsigned, unsigned); };
H8::H8(unsigned a, unsigned b) { mSize = b; mFreeHead = this; mCount = 0; mUsedHead = this; E8& l = mListHead; l.mNext = &l; l.mPrev = &l; AllocatePageBlock(a); }

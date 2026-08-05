struct E { E *mNext; E *mPrev; };
struct S { E mHead; unsigned a; S(unsigned x); };
S::S(unsigned x) { a = x; E &l = mHead; l.mNext = &l; }

struct E { E *mNext; E *mPrev; };
struct S { E mHead; S(); };
S::S() { E &l = mHead; l.mNext = &l; }

struct BE { BE* mNext; BE* mPrev; };
struct P {
  P(unsigned int a, unsigned int b);
  void lf(unsigned int a, unsigned int b);
  BE* Alloc(unsigned int);
  BE* Reset();
  unsigned int m0; unsigned int m1; unsigned int m2; unsigned int m3;
  P* m4; P* m5;
};
P::P(unsigned int a, unsigned int b) {
  m0 = 0;
  m1 = b;
  m2 = a;
  Reset();
}

struct BE { BE* mNext; BE* mPrev; };
struct Q {
  Q(unsigned int a, unsigned int b, unsigned int c);
  void lf(unsigned int a, unsigned int b, unsigned int c);
  BE* A2(unsigned int, unsigned int);
  BE* A1(unsigned int);
  BE* A0();
  Q* n0; Q* n1;
  unsigned int k0; unsigned int k1; unsigned int k2; unsigned int k3;
  unsigned int k4; unsigned int k5;
};
Q::Q(unsigned int a, unsigned int b, unsigned int c) {
  k0 = 0;
  n0 = this;
  n1 = this;
  A2(a, b);
}

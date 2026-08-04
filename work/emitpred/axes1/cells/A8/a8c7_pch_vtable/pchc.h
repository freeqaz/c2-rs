#ifndef PCHC_H
#define PCHC_H
struct C {
  int f;
  C() : f(1) {}
  virtual ~C() {}
  virtual int v(int x) { return x + f; }
  virtual int w(int x) { return x - f; }
};
#endif

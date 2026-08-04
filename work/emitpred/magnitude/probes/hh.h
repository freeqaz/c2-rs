#ifndef HH_H
#define HH_H
struct B {
  int b;
  B() : b(0) {}
  virtual ~B() {}
  virtual int bv(int x) { return x + b; }
};
struct C : B {
  int f;
  C() : f(1) {}
  virtual ~C() {}
  virtual int v(int x) { return x + f; }
  virtual int w(int x) { return x - f; }
  virtual int u(int x) { return x * f; }
  int nv(int x) { return x * 2 + f; }
};
struct Q { virtual ~Q() {} virtual int q(int) ; };
struct MI : Q, C { virtual int q(int x) { return x+2; } virtual int v(int x) { return x+3; } };
#endif

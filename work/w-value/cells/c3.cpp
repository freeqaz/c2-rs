struct Obj { int Get(); void Set(int); };
int b1(Obj *p, int a) { return a == p->Get(); }
int b2(Obj *p, int a, int b) { return a + p->Get() / b; }
int b3(Obj *p, int a) { return a < p->Get(); }
int b4(Obj *p, int a) { return a + p->Get(); }
int b5(Obj *p, int a) { return a + (p->Get() != 0); }
int b6(Obj *p, unsigned a) { return a % p->Get(); }

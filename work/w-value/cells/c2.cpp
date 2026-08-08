struct Obj { int Get(); void Set(int); };
int a1(Obj *p, int a) { return a + p->Get(); }
int a2(Obj *p, int *q) { return *q + p->Get(); }
int a3(Obj *p, int a) { return a == p->Get(); }
int a4(Obj *p, Obj *r) { return r->Get() + p->Get(); }
int a5(Obj *p, int a) { return a + p->Get() + a; }
void a6(Obj *p, int a) { p->Set(a); }

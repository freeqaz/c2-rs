struct Obj { int Get(); void Set(int); };
struct V { int x; int G(); };
V mk();
extern int gI;
extern Obj gO;
int h(const char *);
int n1(Obj *p, int a) { return a + p->Get(); }
int n2(Obj *p, int a, int b) { int t = a; if (b + p->Get()) t = b; return t; }
int n3(int a) { return a + mk().G(); }
int n4(int a) { return a + gI; }
int n5(int a) { return a + h("hi"); }
int n6(int a) { return a + gO.Get(); }

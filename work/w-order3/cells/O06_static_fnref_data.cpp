struct A{int a;};
static A g;
A* p = &g;
void f(){ g.a = 1; }

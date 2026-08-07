struct A{int a;};
A g;
A* p = &g;
void f(){ g.a = 1; }

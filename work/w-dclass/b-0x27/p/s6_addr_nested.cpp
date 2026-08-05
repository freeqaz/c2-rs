struct T { unsigned x; unsigned y; };
struct S { unsigned a; T t; };
unsigned *f(S *p) { return &p->t.y; }

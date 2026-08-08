struct S { int m; void set(int); int get(); void none(); };
void c1(S* s, int a) { s->set(a); }
void c2_(S* s, int a) { s->set(a); s->set(a); }
void c3(S* s, int a) { s->m = a; s->set(a); }
int  c4(S* s) { return s->get(); }
void c5(S* s, S* t, int a) { s->set(a); t->set(a); }
void c6(S* s) { s->none(); }
void c7(S* s, int a) { s->set(a); s->m = a; }
int  c8(S* s, int a) { s->set(a); return a; }

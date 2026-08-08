struct S { int m; void a(); void b(); void set(int); };
void g1();
void g2();

// the FREE-function sequence the port already emits (Class A)
void free2() { g1(); g2(); }

// the member-call sequence: receiver is a formal, live across the first call
void mem2(S* s) { s->a(); s->b(); }
void mem3(S* s) { s->a(); s->b(); s->a(); }
void mem2arg(S* s, int x) { s->set(x); s->set(x); }
void mem2two(S* s, S* t) { s->a(); t->a(); }
void memfree(S* s) { s->a(); g1(); }
void freemem(S* s) { g1(); s->a(); }

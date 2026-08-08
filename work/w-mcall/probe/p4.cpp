struct S {
    int m;
    void a();
    void b();
    void set(int);
    int get();
    void both();
};
struct V { virtual void v(); };
struct L { L *Next(); void Val(); };
void gfree();
extern S gO;

void P1(S *s) { s->a(); s->b(); }
void P2(S *s) { s->a(); s->b(); s->a(); }
void P3(S *s, int x) { s->set(x); s->set(x); }
void P4(S *s, S *t) { s->a(); t->a(); }
void P5(S *s) { s->a(); gfree(); }
void P6(S *s) { gfree(); s->a(); }
int P7(S *s) { s->a(); return 5; }
void S::both() { a(); b(); }

void N1() { gO.a(); gO.b(); }
int N2(S *s) { s->a(); return s->get(); }
void N3(S *s, int c) { if (c) { s->a(); } s->b(); }
void N4(S *s, S *t, int a) { s->set(a); t->set(a); }
void N5(V *v) { v->v(); v->v(); }
void N6(S *s) { { s->a(); s->b(); } }
void N7(L *l) { l->Next()->Val(); gfree(); }

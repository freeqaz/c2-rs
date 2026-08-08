struct S { void a(); void b(); void set(int); float f(); };
void g1(int, int);
void g2(int, int);
// the FREE twin of the three-values-live member cell
void t3(int a, int b, int c) { g1(a, c); g2(b, c); }
// a float-returning member call whose result is discarded
void tfp(S *s) { s->f(); s->a(); }
// more calls than MAX_SEQ_CALLS
void tlong(S *s) {
    s->a(); s->b(); s->a(); s->b(); s->a(); s->b();
    s->a(); s->b(); s->a(); s->b(); s->a(); s->b();
    s->a(); s->b(); s->a(); s->b(); s->a(); s->b();
}
// nine formals
void t9(S *s, int a, int b, int c, int d, int e, int f, int g, int h) {
    s->set(a + b + c + d + e + f + g + h);
    s->a();
}

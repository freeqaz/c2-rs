// M1 = mm + the ENTRY-BLOCK PARK: the call's arguments are the two formals in
// the OPPOSITE order, which is what forces `mr r11,r3 ; mr r3,r4`.
void g(void *, void *);
int f(void *p, void *q) { if (p == 0) return 5; if (q == 0) return 11; g(q, p); return 0; }

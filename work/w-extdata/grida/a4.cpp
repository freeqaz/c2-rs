extern int gI;
extern int gJ;
extern void g1(int *);
extern void g2(int *);
void a4(void) { g1(&gI); g2(&gJ); }

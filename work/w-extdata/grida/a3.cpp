extern int gI;
extern int gJ;
extern void g0(void);
extern void g1(int *);
extern void g2(int *);
void a3(void) { g1(&gI); g0(); g2(&gJ); }

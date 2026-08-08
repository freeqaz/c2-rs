extern int gI;
extern void g0(void);
extern void g1(int *);
void a1(void) { g1(&gI); g0(); }

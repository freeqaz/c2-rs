extern int gI;
extern void g0(void);
extern void g1(int *);
void a2(void) { g0(); g1(&gI); }

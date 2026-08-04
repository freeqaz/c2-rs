extern void v0(void);
extern void v1(void);
extern void v2(void);
void A(int a) { if (a != 0) v0(); v1(); }
void B(int a) { if (a != 0) v0(); else v1(); v2(); }

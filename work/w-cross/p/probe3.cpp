extern void v0(void);
extern void v1(void);
extern void v2(void);
extern void a1(int);
extern void a2(int, int);
extern int  i0(void);
extern int  i1(int);
extern int  i2(int, int);

// --- label-counter stride vs BRANCH-TARGET count, framed, everything else held
void L0(void)          { v0(); v1(); }                          // 0 targets
void L1(int a)         { if (a != 0) v0(); v1(); }              // 1 target
void L2(int a)         { if (a != 0) v0(); else v1(); v2(); }   // 2 targets
void L3(int a, int b)  { if (a != 0) v0(); else v1();
                         if (b != 0) v0(); else v2(); v1(); }   // 4 targets
void L4(void)          { v0(); v1(); }                          // 0 targets again

// --- the PARK register when r11 is occupied by a local ---------------------
int  P0(int a, int b, int c) { int r = 0; if (a != 0) r = i2(b, c); return r; }
int  P1(int a, int b)        { int r = 0; if (a != 0) r = i1(b); return r; }
void P2(int a, int b, int c) { if (a != 0) a2(b, c); a1(a); }

// --- a saved GPR AND a branch ----------------------------------------------
void S0(int a, int b)  { if (a != 0) v0(); a1(b); }
int  S1(int a, int b)  { if (a != 0) v0(); return i1(b); }

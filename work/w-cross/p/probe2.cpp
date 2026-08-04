extern void v0(void);
extern void v1(void);
extern void a1(int);
extern void a2(int, int);
extern int  i0(void);
extern int  i1(int);

// --- where does the guarded call's SETUP go, relative to the compare? -------
void s0(int a)          { if (a != 0) v0(); v1(); }            // no setup
void s1(int a, int b)   { if (a != 0) a1(b); v1(); }           // setup in the arm
void s2(int a, int b)   { if (a != 0) a1(b); a1(a); }          // both calls have setup
void s3(int a, int b)   { if (b != 0) a1(a); v1(); }           // scrutinee is NOT r3
void s4(int a, int b)   { if (a != 0) a2(b, a); v1(); }        // two args in the arm

// --- three blocks: the intra-section `b` -----------------------------------
void t0(int a)          { if (a != 0) v0(); else v1(); v0(); } // if/else + trailer
void t1(int a, int b)   { if (a != 0) a1(b); else a1(a); v1(); }

// --- a value live across a call --------------------------------------------
int  u0(int a)          { int r = 0; if (a != 0) r = i0(); return r; }
int  u1(int a, int b)   { int r = 0; if (a != 0) r = i1(b); return r; }
int  u2(int a, int b)   { int r = 0; if (a != 0) r = i1(b); else r = i1(a); return r; }

// --- the guard is a bare value, not a comparison ---------------------------
void b0(int a)          { if (a) v0(); v1(); }
void b1(void *p)        { if (p) v0(); v1(); }

// --- the arm returns early (band-2 hazard next door) -----------------------
void e0(int a)          { if (a != 0) return; v0(); }

// W11 axis cross. STRUCTURAL axes first (guard count, arm content, result kind,
// trailing-call count, scrutinee position, literal identity between exits);
// values (the relation, the literal magnitude) varied inside the cells.
void v0(); void v1(); void v2(); void v3();

// --- axis: guard COUNT (1,2,3) with distinct literals ----------------------
int g1(int a)               { if (a) return 5; v0(); return 0; }
int g2(int a,int b)         { if (a) return 5; if (b) return 11; v0(); return 0; }
int g3(int a,int b,int c)   { if (a) return 5; if (b) return 11; if (c) return 22; v0(); return 0; }

// --- axis: literal IDENTITY between two exits (tail-merge hazard, #193) ----
int m2(int a,int b)         { if (a) return 5; if (b) return 5; v0(); return 0; }
int m0(int a)               { if (a) return 0; v0(); return 0; }

// --- axis: trailing-call COUNT --------------------------------------------
int t2(int a)               { if (a) return 5; v0(); v1(); return 0; }
int t4(int a)               { if (a) return 5; v0(); v1(); v2(); v3(); return 0; }

// --- axis: scrutinee POSITION (3rd formal) --------------------------------
int p3(int a,int b,int c)   { if (c) return 5; v0(); return 0; }

// --- axis: RESULT kind ----------------------------------------------------
void rv(int a)              { if (a) return; v0(); v1(); }
int rn(int a)               { if (a) return -1; v0(); return 32767; }

// --- axis: RELATION / signedness (values, inside one structural cell) -----
int r_ne(int a)             { if (a != 0) return 5; v0(); return 0; }
int r_lt(int a)             { if (a < 3) return 5; v0(); return 0; }
int r_ge(unsigned a)        { if (a >= 7u) return 5; v0(); return 0; }
int r_eq(void* p)           { if (p == 0) return 5; v0(); return 0; }

// --- axis: ARM content (a call inside the arm) ----------------------------
int ac(int a)               { if (a) { v1(); return 5; } v0(); return 0; }
// --- control: no guard at all (the shipped CallSeq with an int result) ----
int c0()                    { v0(); v1(); return 0; }

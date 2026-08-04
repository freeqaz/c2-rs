void v0(); void v1();
// void result, N guards — does the empty-arm inversion compose?
void w1(int a)                 { if (a != 0) return; v0(); v1(); }
void w2(int a,int b)           { if (a != 0) return; if (b != 0) return; v0(); v1(); }
void w3(int a,int b,int c)     { if (a != 0) return; if (b != 0) return; if (c != 0) return; v0(); }
// int result, guard on the 4th formal (r6) and an unsigned non-zero literal
int  x4(int a,int b,int c,unsigned d) { if (d >= 7u) return 5; v0(); return 0; }
// int result, three guards with a NEGATIVE literal exit and a wide-ish one
int  x5(int a,int b)           { if (a < -3) return -1; if (b > 100) return 4660; v0(); return 0; }
// the boundary: a guard whose arm falls THROUGH (W10's shape) beside an early return
int  x6(int a,int b)           { if (a != 0) return 5; if (b != 0) v0(); v1(); return 0; }

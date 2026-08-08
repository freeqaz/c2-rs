void g1(int);
void g2(int);
void g0();
// the free-function analogues of the member-call cells, to check the emitter
// already lowers the Class B shapes the receiver will produce.
void s1(int a) { g1(a); g2(a); }
void s2(int a, int b) { g1(a); g2(b); }
void s3(int a) { g1(a); g0(); }

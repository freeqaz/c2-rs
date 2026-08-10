// P4 — one eager EXTERNAL .bss object and a FRAMED function.
unsigned int g1;
extern void sink(int);
void F(int a) { sink(a); g1 = a; }

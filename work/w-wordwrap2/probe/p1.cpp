// P1 — one eager EXTERNAL .bss object, one leaf function that stores to it.
unsigned int g1;
void S1(unsigned int x) { g1 = x; }

// P2 — TWO eager EXTERNAL .bss objects, one function each.
unsigned int g1;
unsigned int g2;
void S1(unsigned int x) { g1 = x; }
void S2(unsigned int x) { g2 = x; }

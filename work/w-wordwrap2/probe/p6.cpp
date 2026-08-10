// P6 — an eager EXTERNAL .bss object BESIDE an initialized .data object.
unsigned int g1;
unsigned int d1 = 7;
void S1(unsigned int x) { g1 = x; }
void S2(unsigned int x) { d1 = x; }

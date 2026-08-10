// P8 — a .bss object of a size/alignment pair that PROMOTES (n >= 64 -> 8).
unsigned int big[32];
unsigned int g1;
void S1(unsigned int x) { big[0] = x; }
void S2(unsigned int x) { g1 = x; }

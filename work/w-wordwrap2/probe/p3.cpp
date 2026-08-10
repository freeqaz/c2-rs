// P3 — TWO eager EXTERNAL .bss objects, ONE function referencing BOTH
// (wordwrap's ?WordWrap_CanBreakLineAt shape at n = 2).
unsigned int g1;
unsigned int g2;
void S(unsigned int x) { g1 = x; g2 = x; }

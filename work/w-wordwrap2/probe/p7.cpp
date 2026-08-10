// P7 — THREE eager EXTERNAL .bss objects: above MAX_OBJECTS_PER_SECTION.
unsigned int g1;
unsigned int g2;
unsigned int g3;
void S1(unsigned int x) { g1 = x; }
void S2(unsigned int x) { g2 = x; }
void S3(unsigned int x) { g3 = x; }

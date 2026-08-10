// P5 — an INTERNAL-linkage object first reached from a FUNCTION BODY: slot C.
static unsigned int s1;
void S1(unsigned int x) { s1 = x; }
unsigned int R() { return s1; }

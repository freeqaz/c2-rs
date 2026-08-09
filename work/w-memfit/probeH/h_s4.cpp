// w-memfit alignment-hint probe s4
struct S4 { int a; };
extern "C" void *memcpy(void *, const void *, unsigned int);
void f(S4 *d, const S4 *s) { memcpy(d, s, 96); }

// w-memfit alignment-hint probe s16
struct S16 { double a; double b; };
extern "C" void *memcpy(void *, const void *, unsigned int);
void f(S16 *d, const S16 *s) { memcpy(d, s, 96); }

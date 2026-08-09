// w-memfit alignment-hint probe s32
struct S32 { double a[4]; };
extern "C" void *memcpy(void *, const void *, unsigned int);
void f(S32 *d, const S32 *s) { memcpy(d, s, 96); }

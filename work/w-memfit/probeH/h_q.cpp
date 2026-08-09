// w-memfit alignment-hint probe q

extern "C" void *memcpy(void *, const void *, unsigned int);
void f(long long *d, const long long *s) { memcpy(d, s, 96); }

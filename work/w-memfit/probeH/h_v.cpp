// w-memfit alignment-hint probe v

extern "C" void *memcpy(void *, const void *, unsigned int);
void f(void *d, const void *s) { memcpy(d, s, 96); }

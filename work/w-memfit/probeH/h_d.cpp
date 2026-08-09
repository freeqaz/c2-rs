// w-memfit alignment-hint probe d

extern "C" void *memcpy(void *, const void *, unsigned int);
void f(double *d, const double *s) { memcpy(d, s, 96); }

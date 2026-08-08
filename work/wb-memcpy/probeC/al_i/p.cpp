extern "C" void *memcpy(void *, const void *, unsigned int);
void f(int *d, const int *s) { memcpy(d, s, 96); }

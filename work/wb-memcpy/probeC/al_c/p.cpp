extern "C" void *memcpy(void *, const void *, unsigned int);
void f(char *d, const char *s) { memcpy(d, s, 96); }

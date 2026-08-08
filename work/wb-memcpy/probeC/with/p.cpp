// wb-memcpy probe C — the copy is present and BOTH operands are dead locals
extern "C" void *memcpy(void *, const void *, unsigned int);
extern "C" void sink(void *);
void f(int k) {
    double a[32]; double b[32]; (void)k;
    memcpy(a, b, 96);
}

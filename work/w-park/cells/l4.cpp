extern "C" void *memcpy(void *, const void *, unsigned int);
unsigned long L4(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    memcpy(a1, a0, 0x48);
    return 0;
}

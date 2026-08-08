void g3n(void *, void *, unsigned int);
unsigned long L3(void *a0, void *a1, unsigned int a2) {
    if (a0 == 0) return 5;
    if (a1 == 0) return 11;
    g3n(a1, a0, 0x48);
    return 0;
}

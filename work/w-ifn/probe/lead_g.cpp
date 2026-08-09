int gz(int);
extern "C" void *memcpy(void *, const void *, unsigned int);
extern "C" long subject(void *h, void *p, unsigned int f) {
    if (h == 0) return 5;
    if (p == 0) return 11;
    memcpy(p, h, 0x48);
    return 0;
}
int framed(int a) { return gz(a) + 7; }

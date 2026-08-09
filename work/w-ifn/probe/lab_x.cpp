typedef unsigned int uint;
extern "C" void *memcpy(void *, const void *, unsigned int);
int gz(int);
int framed(int a) { return gz(a) + 7; }
extern "C" long sub(void *h, void *p, uint f) {
    if (h == 0) { return 5; }
    if (p == 0) { return 11; }
    memcpy(p, h, 0x48);
    return 0;
}

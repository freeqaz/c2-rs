int gz(int);
typedef unsigned int uint;
extern "C" void FreeHandleL(void *);
struct Info { unsigned a, b; uint (*proc)(void *, uint, uint, uint); };
extern "C" __declspec(noinline) uint kflush(void *h, uint f) { return 0; }
extern "C" __declspec(noinline) uint kbuf(void *h, char *b, long c, uint f) { return 0; }
extern "C" uint subject(void *h, uint c) {
    if (h == 0) return 5;
    uint r = kflush(h, 0);
    if (r != 0) return r;
    Info *i = (Info *)h;
    uint s = i->proc(i, 4, c, 0);
    if (s != 0) return s;
    kbuf(h, 0, 0, 0);
    FreeHandleL(h);
    return 0;
}
int framed(int a) { return gz(a) + 7; }

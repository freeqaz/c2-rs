int gz(int);
typedef unsigned int uint;
extern "C" __declspec(noinline) uint kflush(void *h, uint f) { return 0; }
extern "C" __declspec(noinline) uint kbuf(void *h, char *b, long c, uint f) { return 0; }
int leaf_none(int a) { return a + 1; }
int framed(int a) { return gz(a) + 7; }

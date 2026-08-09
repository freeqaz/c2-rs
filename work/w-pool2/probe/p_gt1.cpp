#include "Pool.h"

Pool::Pool(int i1, void *v, int i2) : mFree((char *)v) {
    char *ptr = (char *)v;
    int stride = (i1 + 3) & ~3;
    int count = i2 / stride;
    if (count > 1) {
        int n = count - 1;
        do {
            char *next = ptr + stride;
            *(char **)ptr = next;
            ptr = next;
        } while (--n);
    }
    *(char **)ptr = 0;
}

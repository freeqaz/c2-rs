#include "Pool.h"

void *Pool::Alloc() {
    void *ptr = mFree;
    if (!ptr)
        return (void *)1;
    mFree = *(char **)ptr;
    return ptr;
}

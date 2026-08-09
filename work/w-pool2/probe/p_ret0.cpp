#include "Pool.h"

void *Pool::Alloc() {
    void *ptr = mFree;
    if (!ptr)
        return nullptr;
    mFree = *(char **)ptr;
    return ptr;
}

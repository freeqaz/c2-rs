struct P { char *mFree; };
void *wpool_alloc(P *p) {
    void *ptr = p->mFree;
    if (!ptr) return 0;
    p->mFree = *(char **)ptr;
    return ptr;
}

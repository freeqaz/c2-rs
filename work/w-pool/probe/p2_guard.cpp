struct P { char *mFree; };
void wpool_free(P *p, void *v) {
    if (!v) return;
    *(void **)v = p->mFree;
    p->mFree = (char *)v;
}

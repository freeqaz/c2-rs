struct P { char *mFree; };
void wpool_free_noguard(P *p, void *v) {
    *(void **)v = p->mFree;
    p->mFree = (char *)v;
}

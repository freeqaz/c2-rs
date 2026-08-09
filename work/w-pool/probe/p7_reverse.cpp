struct P { char *mFree; };
void wpool_reverse(P *p, void *v) {
    p->mFree = (char *)v;
    *(void **)v = p->mFree;
}

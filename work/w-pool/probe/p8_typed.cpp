struct P { char *mFree; };
void wpool_typed(P *p, void **v) {
    *v = p->mFree;
    p->mFree = (char *)v;
}
